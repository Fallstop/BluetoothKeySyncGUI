#[app_macros::ipc_type]
#[serde(tag = "type", content = "data")]
pub enum Message<T> {
    Success(T),
    Error(String),
}
