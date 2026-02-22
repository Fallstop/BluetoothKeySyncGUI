mod linux_api;
mod message;
mod sync_api;
mod windows_api;

use crate::api::{linux_api::LinuxApi, sync_api::SyncApi, windows_api::WindowsApi};

use tauri::Runtime;
use taurpc::Router;

pub fn init<R: Runtime>() -> Router<R> {
    let router = Router::new()
        .merge(windows_api::WindowsImpl.into_handler())
        .merge(linux_api::LinuxApiImpl.into_handler())
        .merge(sync_api::SyncApiImpl.into_handler());

    return router;
}
