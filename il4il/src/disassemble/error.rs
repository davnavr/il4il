/// Error type used when printing IL4IL assembly fails.
#[derive(Debug, thiserror::Error)]
#[error("error printing dissassembly: {0}")]
#[repr(transparent)]
pub struct Error(Box<dyn std::error::Error>);

impl Error {
    pub fn new<E: std::error::Error + 'static>(error: E) -> Self {
        Self(Box::from(error))
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::new(error)
    }
}

impl From<std::fmt::Error> for Error {
    fn from(error: std::fmt::Error) -> Self {
        Self::new(error)
    }
}
