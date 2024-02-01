use std::fmt::Debug;
use thiserror::Error;

/// This is the kind of error for the core
#[derive(Error, Debug, Clone, Eq, PartialEq)]
pub enum CoreError {
    #[error("{0}")]
    DataError(String),
    #[error("{0}")]
    ResourceNotFound(String),
    #[error("{0}")]
    OperationNotAuthorized(String),
    #[error("{0}")]
    OperationForbiden(String),
    #[error("{0}")]
    UnkownError(String),
}

/// This trait is used to convert Error to CoreError
/// Example:
/// ```
/// use common::error::CoreError;
/// # use crate::common::error::Error;
/// #[derive(Debug)]
/// pub struct AsCoreError(CoreError);
///
/// impl common::error::Error for AsCoreError {
///     fn get_core_error(&self) -> CoreError {
///         self.0.to_owned()
///     }
/// }
/// # fn main() {
/// let error = AsCoreError(CoreError::UnkownError(String::from("test error")));
/// assert_eq!(error.get_core_error(), CoreError::UnkownError(String::from("test error")));
/// # }
///
/// ```
pub trait Error: Sync + Send + Debug {
    fn get_core_error(&self) -> CoreError;
}
