use core::fmt;
use std::error;
use std::fmt::Formatter;

#[derive(Debug, Clone)]
pub struct ProgramTooLargeError;

impl fmt::Display for ProgramTooLargeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "program too large to fit in memory")
    }
}

impl error::Error for ProgramTooLargeError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}
