
use std::fmt::{Debug, Display, Formatter, Result};
use std::error::Error;

#[derive(Debug)]
pub struct StringError(pub String);

impl Display for StringError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        Debug::fmt(self, f)
    }
}

impl Error for StringError {
    fn description(&self) -> &str {
        &*self.0
    }
}
