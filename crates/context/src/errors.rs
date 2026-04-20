use derive_more::Display;
use std::result;
#[derive(Clone, Debug, Display)]
pub struct PredicateError(pub String);
impl From<&str> for PredicateError {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}
impl std::fmt::Display for PredicateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl From<String> for PredicateError {
    fn from(value: String) -> Self {
        Self(value)
    }
}
pub type PredicateResult<T> = result::Result<T, PredicateError>;

pub type Result<T> = result::Result<T, MockError>;
#[derive(Debug, Clone)]
pub enum MockError {
    NoMatchingId,
    PredicateError(PredicateError),
    Other(String),
}

impl std::fmt::Display for MockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MockError::NoMatchingId => write!(f, "no matching id"),
            MockError::PredicateError(predicate_error) => {
                write!(f, "{predicate_error}")
            }
            MockError::Other(s) => write!(f, "{s}"),
        }
    }
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
