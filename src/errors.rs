use thiserror::Error;

#[derive(Error, Debug)]
pub enum HtmxError {
    #[error("No attributes provided!")]
    EmptyAttrs,
    #[error("Invalid uri provided: {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),
}
