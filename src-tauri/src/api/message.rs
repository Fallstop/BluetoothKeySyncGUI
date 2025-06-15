#[app_macros::ipc_type]
#[serde(tag = "type", content = "data")]
pub enum Message<T> {
    Success(T),
    Error(String),
}

impl<T> Message<T> {
    pub fn from_error<E: std::error::Error>(error: E) -> Self {
        Message::Error(error.to_string())
    }
    pub fn from_result(result: Result<T, impl std::error::Error>) -> Self {
        match result {
            Ok(value) => Message::Success(value),
            Err(error) => Message::from_error(error),
        }
    }
}
