use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Failed to parse. TODO: better error messages")]
    Failed,
}
