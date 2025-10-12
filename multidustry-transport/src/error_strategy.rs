pub enum ErrorStrategy {
    Drop,
    ThrowError,
}

impl Default for ErrorStrategy {
    fn default() -> Self {
        ErrorStrategy::ThrowError
    }
}
