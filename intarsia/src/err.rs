use thiserror::Error;

#[derive(Error, Clone, Debug, PartialEq)]
pub enum Error {
    /// An error that is used if a project exists already.
    #[error("Project exists already!")]
    ExistsAlready,
    #[error("Project does not exist yet!")]
    DoesNotExist,
    #[error("External `{0}`")]
    External(String),
}
