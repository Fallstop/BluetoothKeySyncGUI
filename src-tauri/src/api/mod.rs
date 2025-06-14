mod windows_api;
mod message;

use crate::api::windows_api::WindowsApi;

use tauri::{Runtime};
use taurpc::Router;


pub fn init<R: Runtime>() -> Router<R> {
    let router = Router::new()
		.merge(windows_api::WindowsImpl.into_handler());

    return router;
}
