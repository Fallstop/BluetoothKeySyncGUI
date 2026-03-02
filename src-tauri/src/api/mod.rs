mod linux_api;
pub mod message;
pub mod sync_api;
mod update_api;
mod windows_api;

use crate::api::{
    linux_api::LinuxApi, sync_api::SyncApi, update_api::UpdateApi, windows_api::WindowsApi,
};

use tauri::Runtime;
use taurpc::Router;

pub fn init<R: Runtime>() -> Router<R> {
    Router::new()
        .merge(windows_api::WindowsImpl.into_handler())
        .merge(linux_api::LinuxApiImpl.into_handler())
        .merge(sync_api::SyncApiImpl.into_handler())
        .merge(update_api::UpdateApiImpl.into_handler())
}
