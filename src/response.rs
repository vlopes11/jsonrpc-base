use super::{helpers, Error};
use alloc::{
    format,
    string::{String, ToString},
};
use core::str::FromStr;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC response
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response {
    /// Protocol header
    pub jsonrpc: String,

    /// Result variant representing success
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,

    /// Result variant representing error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Error>,

    /// ID of the request that originated the response
    pub id: Value,
}

impl Response {
    /// Create a new response representing a success
    pub fn ok<I, V>(id: I, value: V) -> Self
    where
        I: Into<Value>,
        V: Into<Value>,
    {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(value.into()),
            error: None,
            id: id.into(),
        }
    }

    /// Create a new response representing an error
    pub fn err<I, E>(id: I, err: E) -> Self
    where
        I: Into<Value>,
        E: Into<Error>,
    {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(err.into()),
            id: id.into(),
        }
    }

    /// Parse a message into the response
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

impl<T, E> From<Response> for Result<T, E>
where
    T: From<Value>,
    E: From<Error>,
{
    fn from(value: Response) -> Self {
        match (value.result, value.error) {
            (Some(result), _) => Ok(result.into()),
            (_, Some(err)) => Err(err.into()),
            (_, _) => Err(Error {
                code: Error::INVALID_REQUEST,
                message: "the provided respose header is invalid".to_string(),
                data: None,
            }
            .into()),
        }
    }
}

impl ToString for Response {
    fn to_string(&self) -> String {
        serde_json::to_string(&self)
            .map(|m| format!("Content-Length: {}\r\n\r\n{}", m.len(), m))
            .unwrap_or_else(|_e| "infallible json conversion".to_string())
    }
}

impl FromStr for Response {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).map(|(json, _)| json)
    }
}
