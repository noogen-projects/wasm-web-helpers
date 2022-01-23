use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Reqwasm error: {0:?}")]
    Reqwasm(#[from] reqwasm::Error),

    #[error("Failure response: status = {0}, body = `{1}`")]
    FailureResponse(u16, String),
}
