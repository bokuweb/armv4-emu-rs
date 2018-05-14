pub mod arm;

use std::error;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum ArmError {
    UnknownError,
}

#[derive(Debug)]
pub enum PipelineStatus {
    Flush,
    Continue,
}

impl error::Error for ArmError {
    fn description(&self) -> &str {
        match *self {
            ArmError::UnknownError => "Unknown ARM error",
        }
    }
}

impl fmt::Display for ArmError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ArmError::UnknownError => write!(f, "Unknown ARM error"),
        }
    }
}
