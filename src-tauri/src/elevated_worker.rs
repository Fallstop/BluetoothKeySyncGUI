use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

use bluetooth_model::worker_protocol::{
    WorkerCommand, WorkerOperation, WorkerReady, WorkerResponse, WorkerResult,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::{oneshot, Mutex};

use crate::elevated::{find_askpass, is_elevated, relative_command_path};

static WORKER: OnceLock<ElevatedWorker> = OnceLock::new();

pub fn get_worker() -> &'static ElevatedWorker {
    WORKER.get_or_init(ElevatedWorker::new)
}

enum WorkerState {
    NotStarted,
    Ready,
    Dead,
}

struct WorkerInner {
    state: WorkerState,
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    reader_handle: Option<tokio::task::JoinHandle<()>>,
    pending: HashMap<u64, oneshot::Sender<WorkerResponse>>,
}

pub struct ElevatedWorker {
    inner: Mutex<WorkerInner>,
    next_id: AtomicU64,
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
            }),
            next_id: AtomicU64::new(1),
        }
    }

    /// Ensure the worker process is running. Spawns it if needed.
    async fn ensure_running(&self) -> Result<(), String> {
        let mut inner = self.inner.lock().await;
        match &inner.state {
            WorkerState::Ready => return Ok(()),
            WorkerState::NotStarted | WorkerState::Dead => {
                self.spawn_locked(&mut inner).await?;
                Ok(())
            }
        }
    }

    async fn spawn_locked(&self, inner: &mut WorkerInner) -> Result<(), String> {
        // Clean up old state
        if let Some(mut child) = inner.child.take() {
            let _ = child.kill().await;
        }
        if let Some(handle) = inner.reader_handle.take() {
            handle.abort();
        }
        inner.pending.clear();

        let path = relative_command_path("elevated_scrapper")?;

        let mut child = if is_elevated() {
            tokio::process::Command::new(&path)
                .args(["serve", "--privileged"])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .map_err(|e| format!("Failed to spawn elevated worker: {}", e))?
        } else {
            // Get password via askpass
            let askpass = find_askpass()
                .ok_or_else(|| "No askpass program found. Cannot authenticate.".to_string())?;

            let askpass_output = tokio::process::Command::new(&askpass)
                .arg("Bluetooth Key Sync needs root access to read Bluetooth keys")
                .output()
                .await
                .map_err(|e| format!("Failed to run askpass ({}): {}", askpass, e))?;

            if !askpass_output.status.success() {
                return Err("Authentication cancelled".to_string());
            }

            let mut proc = tokio::process::Command::new("sudo")
                .arg("-S")
                .arg("--")
                .arg(&path)
                .args(["serve", "--privileged"])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .map_err(|e| format!("Failed to spawn sudo: {}", e))?;

            // Write password to stdin (sudo -S reads it), but do NOT drop stdin
            {
                let stdin = proc
                    .stdin
                    .as_mut()
                    .ok_or("Failed to get stdin for sudo")?;
                stdin
                    .write_all(&askpass_output.stdout)
                    .await
                    .map_err(|e| format!("Failed to write password to sudo: {}", e))?;
                stdin
                    .write_all(b"\n")
                    .await
                    .map_err(|e| format!("Failed to write newline to sudo: {}", e))?;
                stdin
                    .flush()
                    .await
                    .map_err(|e| format!("Failed to flush sudo stdin: {}", e))?;
            }

            proc
        };

        let stdout = child
            .stdout
            .take()
            .ok_or("Failed to capture worker stdout")?;
        let stdin = child
            .stdin
            .take()
            .ok_or("Failed to capture worker stdin")?;

        // Wait for ready signal
        let mut reader = BufReader::new(stdout);
        let mut ready_line = String::new();

        let read_result = tokio::time::timeout(
            std::time::Duration::from_secs(15),
            reader.read_line(&mut ready_line),
        )
        .await;

        match read_result {
            Ok(Ok(0)) => {
                inner.state = WorkerState::Dead;
                return Err("Worker exited before sending ready signal".to_string());
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

    /// Shut down the worker process.
    pub async fn shutdown(&self) {
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
