use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};

use bluetooth_model::worker_protocol::{
    WorkerCommand, WorkerOperation, WorkerReady, WorkerResponse, WorkerResult,
};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::{oneshot, Mutex, Notify};

use crate::elevated::{
    find_askpass, flatpak_host_binary_path, is_elevated, is_flatpak, is_snap,
    relative_command_path,
};

static WORKER: OnceLock<ElevatedWorker> = OnceLock::new();

pub fn get_worker() -> &'static ElevatedWorker {
    WORKER.get_or_init(ElevatedWorker::new)
}

/// Authentication method for privilege escalation.
/// "pkexec" uses polkit (default), "sudo_askpass" uses SSH_ASKPASS + sudo -S.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthMethod {
    Pkexec,
    SudoAskpass,
}

impl AuthMethod {
    pub fn from_str(s: &str) -> Self {
        match s {
            "sudo_askpass" => AuthMethod::SudoAskpass,
            _ => AuthMethod::Pkexec,
        }
    }
}

enum WorkerState {
    NotStarted,
    Spawning,
    Ready,
    Dead,
}

struct WorkerInner {
    state: WorkerState,
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    reader_handle: Option<tokio::task::JoinHandle<()>>,
    pending: HashMap<u64, oneshot::Sender<WorkerResponse>>,
    auth_method: AuthMethod,
}

pub struct ElevatedWorker {
    inner: Mutex<WorkerInner>,
    next_id: AtomicU64,
    /// Shared handle to the spawning child process so shutdown() can kill it
    /// without holding the main lock.
    spawning_child: Arc<Mutex<Option<Child>>>,
    /// Notified when a cancel/shutdown is requested during spawning.
    cancel_notify: Arc<Notify>,
}

impl ElevatedWorker {
    fn new() -> Self {
        Self {
            inner: Mutex::new(WorkerInner {
                state: WorkerState::NotStarted,
                child: None,
                stdin: None,
                reader_handle: None,
                pending: HashMap::new(),
                auth_method: AuthMethod::Pkexec,
            }),
            next_id: AtomicU64::new(1),
            spawning_child: Arc::new(Mutex::new(None)),
            cancel_notify: Arc::new(Notify::new()),
        }
    }

    /// Set the authentication method used for the next spawn.
    pub async fn set_auth_method(&self, method: AuthMethod) {
        let mut inner = self.inner.lock().await;
        inner.auth_method = method;
    }

    /// Ensure the worker process is running. Spawns it if needed.
    async fn ensure_running(&self) -> Result<(), String> {
        {
            let inner = self.inner.lock().await;
            match &inner.state {
                WorkerState::Ready => return Ok(()),
                WorkerState::Spawning => {
                    return Err("Worker is already spawning".to_string())
                }
                WorkerState::NotStarted | WorkerState::Dead => {
                    // Will spawn below after dropping lock
                }
            }
        }
        self.spawn().await
    }

    async fn spawn(&self) -> Result<(), String> {
        // Take the lock briefly to clean up and read config
        let (path, auth_method) = {
            let mut inner = self.inner.lock().await;

            // Clean up old state
            if let Some(mut child) = inner.child.take() {
                let _ = child.kill().await;
            }
            if let Some(handle) = inner.reader_handle.take() {
                handle.abort();
            }
            inner.pending.clear();
            inner.state = WorkerState::Spawning;

            let path = relative_command_path("elevated_scrapper")?;
            let auth_method = inner.auth_method;
            (path, auth_method)
            // Lock is dropped here
        };

        let spawn_result = self.spawn_child(&path, auth_method).await;

        let mut child = match spawn_result {
            Ok(child) => child,
            Err(e) => {
                let mut inner = self.inner.lock().await;
                inner.state = WorkerState::Dead;
                return Err(e);
            }
        };

        // Store child in spawning_child so shutdown() can kill it during
        // the ready-signal wait (which may block while pkexec shows its
        // auth dialog).
        let stdout = child
            .stdout
            .take()
            .ok_or("Failed to capture worker stdout")?;
        let stdin = child
            .stdin
            .take()
            .ok_or("Failed to capture worker stdin")?;
        *self.spawning_child.lock().await = Some(child);

        // Wait for ready signal WITHOUT holding the inner lock so that
        // shutdown() can acquire it and set state to Dead immediately.
        // Also race against cancel_notify so cancellation is instant.
        let mut reader = BufReader::new(stdout);
        let mut ready_line = String::new();

        let cancel = self.cancel_notify.clone();
        let read_result = tokio::select! {
            _ = cancel.notified() => {
                if let Some(mut c) = self.spawning_child.lock().await.take() {
                    let _ = c.kill().await;
                }
                let mut inner = self.inner.lock().await;
                inner.state = WorkerState::Dead;
                return Err("Cancelled".to_string());
            }
            result = tokio::time::timeout(
                std::time::Duration::from_secs(15),
                reader.read_line(&mut ready_line),
            ) => result
        };

        // Take the child back from spawning_child
        let mut child = match self.spawning_child.lock().await.take() {
            Some(c) => c,
            None => {
                // Child was killed by shutdown()
                let mut inner = self.inner.lock().await;
                inner.state = WorkerState::Dead;
                return Err("Cancelled".to_string());
            }
        };

        // Re-acquire the lock to finalize state
        let mut inner = self.inner.lock().await;

        // Check if we were cancelled while waiting
        if matches!(inner.state, WorkerState::Dead) {
            let _ = child.kill().await;
            return Err("Cancelled".to_string());
        }

        match read_result {
            Ok(Ok(0)) => {
                inner.state = WorkerState::Dead;
                // Read stderr for the actual error details
                let mut stderr_msg = String::new();
                if let Some(mut stderr) = child.stderr.take() {
                    let _ = stderr.read_to_string(&mut stderr_msg).await;
                }
                let stderr_msg = stderr_msg.trim();
                let exit_info = match child.try_wait() {
                    Ok(Some(status)) => format!("exit status: {}", status),
                    _ => String::new(),
                };
                let detail = match (stderr_msg.is_empty(), exit_info.is_empty()) {
                    (false, false) => format!(" ({}, {})", stderr_msg, exit_info),
                    (false, true) => format!(" ({})", stderr_msg),
                    (true, false) => format!(" ({})", exit_info),
                    (true, true) => String::new(),
                };
                return Err(format!(
                    "Worker exited before sending ready signal{}",
                    detail
                ));
            }
            Ok(Ok(_)) => {
                let ready: WorkerReady = serde_json::from_str(ready_line.trim()).map_err(|e| {
                    format!(
                        "Failed to parse ready signal: {}. Got: {}",
                        e,
                        ready_line.trim()
                    )
                })?;
                if !ready.ready {
                    inner.state = WorkerState::Dead;
                    return Err("Worker sent ready=false".to_string());
                }
            }
            Ok(Err(e)) => {
                inner.state = WorkerState::Dead;
                return Err(format!("Error reading ready signal: {}", e));
            }
            Err(_) => {
                inner.state = WorkerState::Dead;
                let _ = child.kill().await;
                return Err("Timeout waiting for worker ready signal".to_string());
            }
        }

        // Spawn stdout reader task
        let reader_handle = tokio::spawn(stdout_reader_task(reader));

        inner.child = Some(child);
        inner.stdin = Some(stdin);
        inner.reader_handle = Some(reader_handle);
        inner.state = WorkerState::Ready;

        Ok(())
    }

    /// Spawn the child process. This may block for a long time (pkexec dialog,
    /// askpass prompt). Runs WITHOUT holding the main mutex.
    async fn spawn_child(
        &self,
        path: &std::path::Path,
        auth_method: AuthMethod,
    ) -> Result<Child, String> {
        if is_flatpak() {
            let host_path = flatpak_host_binary_path("elevated_scrapper")
                .map_err(|e| format!("Flatpak: failed to resolve host binary path: {}", e))?;

            return tokio::process::Command::new("flatpak-spawn")
                .args(["--host", "--watch-bus", "pkexec"])
                .arg(&host_path)
                .args(["serve", "--privileged"])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .map_err(|e| format!("Failed to spawn flatpak-spawn: {}", e));
        }

        if is_snap() {
            let pkexec_result = tokio::process::Command::new("pkexec")
                .arg(path)
                .args(["serve", "--privileged"])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn();

            return match pkexec_result {
                Ok(proc) => Ok(proc),
                Err(_) => {
                    eprintln!("Snap: pkexec unavailable, attempting direct execution");
                    tokio::process::Command::new(path)
                        .args(["serve", "--privileged"])
                        .stdin(std::process::Stdio::piped())
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .spawn()
                        .map_err(|e| format!("Failed to spawn elevated worker in snap: {}", e))
                }
            };
        }

        if is_elevated() {
            return tokio::process::Command::new(path)
                .args(["serve", "--privileged"])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .map_err(|e| format!("Failed to spawn elevated worker: {}", e));
        }

        match auth_method {
            AuthMethod::Pkexec => {
                // pkexec shows its own graphical auth dialog via polkit agent.
                // Just spawn it — cancellation is handled in spawn() during the
                // ready-signal wait (pkexec blocks until auth succeeds).
                tokio::process::Command::new("pkexec")
                    .arg(path)
                    .args(["serve", "--privileged"])
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
                    .map_err(|e| format!("Failed to spawn pkexec: {}", e))
            }
            AuthMethod::SudoAskpass => {
                let askpass = find_askpass().ok_or_else(|| {
                    "No askpass program found. Cannot authenticate.".to_string()
                })?;

                let askpass_output = tokio::process::Command::new(&askpass)
                    .arg("Bluetooth Key Sync needs root access to read Bluetooth keys")
                    .output()
                    .await
                    .map_err(|e| format!("Failed to run askpass ({}): {}", askpass, e))?;

                if !askpass_output.status.success() {
                    return Err("Authentication cancelled".to_string());
                }

                // Validate the password with a quick no-op command.
                let mut validate = tokio::process::Command::new("sudo")
                    .args(["-S", "-k", "--", "true"])
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                    .map_err(|e| format!("Failed to validate credentials: {}", e))?;

                if let Some(mut stdin) = validate.stdin.take() {
                    let _ = stdin.write_all(&askpass_output.stdout).await;
                    let _ = stdin.write_all(b"\n").await;
                    let _ = stdin.flush().await;
                }

                let validate_result = tokio::time::timeout(
                    std::time::Duration::from_secs(3),
                    validate.wait(),
                )
                .await;

                match validate_result {
                    Ok(Ok(status)) if status.success() => {}
                    Ok(Ok(_)) => return Err("Incorrect password".to_string()),
                    Ok(Err(e)) => return Err(format!("Failed to validate password: {}", e)),
                    Err(_) => {
                        let _ = validate.kill().await;
                        return Err("Incorrect password".to_string());
                    }
                }

                tokio::process::Command::new("sudo")
                    .arg("--")
                    .arg(path)
                    .args(["serve", "--privileged"])
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
                    .map_err(|e| format!("Failed to spawn sudo: {}", e))
            }
        }
    }

    /// Send a command to the worker and wait for the response.
    pub async fn send_command(&self, op: WorkerOperation) -> Result<WorkerResponse, String> {
        self.ensure_running().await?;

        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let cmd = WorkerCommand { id, op };

        let (tx, rx) = oneshot::channel();

        let mut inner = self.inner.lock().await;

        // Check state is still ready
        if !matches!(inner.state, WorkerState::Ready) {
            return Err("Worker is not ready".to_string());
        }

        // Register pending response
        inner.pending.insert(id, tx);

        // Serialize and write command
        let mut json = serde_json::to_string(&cmd)
            .map_err(|e| format!("Failed to serialize command: {}", e))?;
        json.push('\n');

        let write_result = {
            let stdin = inner
                .stdin
                .as_mut()
                .ok_or("Worker stdin not available")?;
            let w = stdin.write_all(json.as_bytes()).await;
            if w.is_ok() {
                stdin.flush().await
            } else {
                w
            }
        };

        if let Err(e) = write_result {
            inner.state = WorkerState::Dead;
            return Err(format!("Failed to write command to worker: {}", e));
        }

        // Drop lock before waiting for response
        drop(inner);

        // Wait for response with timeout
        let resp = tokio::time::timeout(std::time::Duration::from_secs(60), rx)
            .await
            .map_err(|_| "Timeout waiting for worker response".to_string())?
            .map_err(|_| "Worker response channel closed".to_string())?;

        Ok(resp)
    }

    /// Shut down the worker process and cancel any in-progress spawn.
    pub async fn shutdown(&self) {
        // First, kill any in-progress spawning child (doesn't need the main lock)
        if let Some(mut child) = self.spawning_child.lock().await.take() {
            let _ = child.kill().await;
        }
        // Signal cancellation to any waiting spawn_child
        self.cancel_notify.notify_waiters();

        let mut inner = self.inner.lock().await;

        // Try to send shutdown command gracefully
        if matches!(inner.state, WorkerState::Ready) {
            if let Some(stdin) = inner.stdin.as_mut() {
                let cmd = WorkerCommand {
                    id: 0,
                    op: WorkerOperation::Shutdown,
                };
                if let Ok(json) = serde_json::to_string(&cmd) {
                    let _ = stdin.write_all(json.as_bytes()).await;
                    let _ = stdin.write_all(b"\n").await;
                    let _ = stdin.flush().await;
                }
            }
        }

        // Kill the child
        if let Some(mut child) = inner.child.take() {
            let _ = child.kill().await;
        }
        if let Some(handle) = inner.reader_handle.take() {
            handle.abort();
        }
        inner.stdin = None;
        inner.pending.clear();
        inner.state = WorkerState::Dead;
    }
}

/// Background task that reads JSON lines from the worker's stdout and routes
/// responses to the correct oneshot channel.
async fn stdout_reader_task(mut reader: BufReader<ChildStdout>) {
    let worker = get_worker();

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // EOF — worker process exited
                let mut inner = worker.inner.lock().await;
                inner.state = WorkerState::Dead;
                for (_, tx) in inner.pending.drain() {
                    let _ = tx.send(WorkerResponse {
                        id: 0,
                        result: WorkerResult::Err {
                            message: "Worker process exited".to_string(),
                        },
                    });
                }
                break;
            }
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                match serde_json::from_str::<WorkerResponse>(trimmed) {
                    Ok(resp) => {
                        let mut inner = worker.inner.lock().await;
                        if let Some(tx) = inner.pending.remove(&resp.id) {
                            let _ = tx.send(resp);
                        } else {
                            eprintln!(
                                "Received response for unknown command id: {}",
                                resp.id
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "Failed to parse worker response: {}. Line: {}",
                            e, trimmed
                        );
                    }
                }
            }
            Err(e) => {
                let mut inner = worker.inner.lock().await;
                inner.state = WorkerState::Dead;
                for (_, tx) in inner.pending.drain() {
                    let _ = tx.send(WorkerResponse {
                        id: 0,
                        result: WorkerResult::Err {
                            message: format!("Worker read error: {}", e),
                        },
                    });
                }
                break;
            }
        }
    }
}
