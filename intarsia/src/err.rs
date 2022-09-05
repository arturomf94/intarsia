use thiserror::Error;

/// General error enum used for the crate. This wraps all kinds
/// of errors in the intarsia project.
#[derive(Error, Clone, Debug, PartialEq)]
pub enum Error {
    /// Project exists already.
    #[error("Project exists already!")]
    ExistsAlready,
    /// Project does not exist yet.
    #[error("Project does not exist yet!")]
    DoesNotExist,
    /// A "generic" error. This is used for reading/saving
    /// errors or other type of errors that don't fall into any
    /// of the other categories.
    #[error("`{0}`")]
    Generic(String),
    /// The provided path for the project is not valid.
    #[error("Invalid project path: `{0}`")]
    InvalidProjectPath(String),
    /// An image cannot be decoded.
    #[error("Encountered a decoding error whilst reading the image: `{0}`")]
    DecodingError(String),
    /// The project contains no original image.
    #[error("There is no original image to show in this project!")]
    EmptyOriginal,
    /// The project contains no processed image.
    #[error("There is no processed image to show in this project!")]
    EmptyProcessed,
    /// An image cannot be displayed. This is usually raised if
    /// the excecution environment cannot run the `open`
    /// command.
    #[error("Could not show image. It is probable that the execution environment does not support the `open` command: `{0}`")]
    OpenFailure(String),
    /// Could not reduce the number of colours in an image.
    #[error("Could not reduce the colours of this image: `{0}`")]
    ColourReductionErr(String),
    /// An error associated to the plotting of an image.
    #[error("Encountered a plotting error: `{0}`")]
    PlotterError(String),
}
