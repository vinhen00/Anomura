use derive_more::Display;
use std::result;
#[derive(Clone, Debug, Display)]
pub struct PredicateError(pub String);
impl From<&str> for PredicateError {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}
impl From<String> for PredicateError {
    fn from(value: String) -> Self {
        Self(value)
    }
}
pub type PredicateResult<T> = result::Result<T, PredicateError>;

pub type Result<T> = result::Result<T, MockError>;
#[derive(Debug, Clone, Display)]
pub enum MockError {
    NoMatchingId,
    PredicateError(PredicateError),
    Other(String),
}

impl From<String> for MockError {
    fn from(value: String) -> Self {
        MockError::Other(value)
    }
}
impl From<&str> for MockError {
    fn from(value: &str) -> Self {
        MockError::Other(value.into())
    }
}
