use alloc::string::String;
use core::fmt;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC protocol error.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Error {
    /// Error code
    pub code: i32,
    /// Human-friendly message
    pub message: String,
    /// Underlying data
    pub data: Option<Value>,
}

impl Error {
    /// Protocol level parse error reserved code
    pub const PARSE_ERROR: i32 = -32700;
    /// Protocol level invalid request reserved code
    pub const INVALID_REQUEST: i32 = -32600;
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
