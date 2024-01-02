use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    // #[error(transparent)]
    // Other(#[from] anyhow::Error),
}
