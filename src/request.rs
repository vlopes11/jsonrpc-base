use super::{helpers, Error};
use alloc::{
    format,
    string::{String, ToString},
};
use core::str::FromStr;
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};

/// JSON-RPC request
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Request {
    /// Protocol header
    pub jsonrpc: String,
    /// Request ID
    pub id: Value,
    /// Method name
    pub method: String,

    /// Optional method arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl Request {
    /// Create a new request with the provided method.
    ///
    /// Will generate a random ID, if the feature `uuid` is enabled. Otherwise, set the ID to `0`.
    pub fn new<M>(method: M) -> Self
    where
        M: ToString,
    {
        #[cfg(feature = "uuid")]
        let id = Value::String(uuid::Uuid::new_v4().to_string());

        #[cfg(not(feature = "uuid"))]
        let id = Value::Number(0.into());

        Request {
            id,
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: None,
        }
    }

    /// Replace the method ID with the provided numeric value
    pub fn with_id<I>(mut self, id: I) -> Self
    where
        I: Into<Number>,
    {
        self.id = Value::Number(id.into());
        self
    }

    /// Replace the method ID with the provided string
    pub fn with_id_string<I>(mut self, id: I) -> Self
    where
        I: ToString,
    {
        self.id = Value::String(id.to_string());
        self
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

    /// Split the request into its ID and message
    pub fn prepare(&self) -> (Value, String) {
        let id = self.id.clone();
        let message = self.to_string();
        (id, message)
    }

    /// Parse a message into the request
    pub fn parse(s: &str) -> Result<(Self, &str), Error> {
        let (length, message) = helpers::get_content_length(s)?;
        if message.len() < length {
            return Err(Error {
                code: Error::INVALID_REQUEST,
                message: "the provided request is invalid".to_string(),
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

impl FromStr for Request {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).map(|(json, _)| json)
    }
}

impl ToString for Request {
    fn to_string(&self) -> String {
        serde_json::to_string(&self)
            .map(|m| format!("Content-Length: {}\r\n\r\n{}", m.len(), m))
            .unwrap_or_else(|_e| "infallible json conversion".to_string())
    }
}
