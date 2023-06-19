use super::{helpers, Error};
use alloc::{
    format,
    string::{String, ToString},
};
use core::str::FromStr;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC notification
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Notification {
    /// Protocol header
    pub jsonrpc: String,
    /// Method name
    pub method: String,

    /// Optional method arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl Notification {
    /// Create a new notification with the provided method
    pub fn new<M>(method: M) -> Self
    where
        M: ToString,
    {
        Notification {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: None,
        }
    }

    /// Replace the methods arguments with the provided value
    pub fn with_params<P>(self, params: P) -> Result<Self, Error>
    where
        P: Serialize,
    {
        serde_json::to_value(params)
            .map_err(|e| Error {
                code: Error::PARSE_ERROR,
                message: e.to_string(),
                data: None,
            })
            .map(|params| self.with_params_value(params))
    }

    /// Replace the methods arguments with the parsed value
    pub fn with_params_value(mut self, params: Value) -> Self {
        self.params = Some(params);
        self
    }

    /// Parse a message into the notification
    pub fn parse(s: &str) -> Result<(Self, &str), Error> {
        let (length, message) = helpers::get_content_length(s)?;
        if message.len() < length {
            return Err(Error {
                code: Error::INVALID_REQUEST,
                message: "the provided request header is invalid".to_string(),
                data: Some(Value::String(s.to_string())),
            });
        }
        let (message, rest) = message.split_at(length);
        let json = serde_json::from_str(message).map_err(|e| Error {
            code: Error::INVALID_REQUEST,
            message: e.to_string(),
            data: Some(Value::String(s.to_string())),
        })?;
        Ok((json, rest))
    }
}

impl FromStr for Notification {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).map(|(json, _)| json)
    }
}

impl ToString for Notification {
    fn to_string(&self) -> String {
        serde_json::to_string(&self)
            .map(|m| format!("Content-Length: {}\r\n\r\n{}", m.len(), m))
            .unwrap_or_else(|_e| "infallible json conversion".to_string())
    }
}
