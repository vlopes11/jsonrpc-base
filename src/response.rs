use super::{helpers, Error};
use alloc::string::{String, ToString};
use core::{fmt, str::FromStr};
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
        let (message, remainder) = helpers::get_content_length(s)?;
        let response = Response::parse_json(message)?;
        Ok((response, remainder))
    }

    /// Parse a response from the provided JSON
    pub fn parse_json(json: &str) -> Result<Self, Error> {
        serde_json::from_str(json).map_err(|e| Error {
            code: Error::INVALID_REQUEST,
            message: e.to_string(),
            data: Some(Value::String(json.to_string())),
        })
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

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        serde_json::to_string(&self)
            .map_err(|_| fmt::Error)
            .and_then(|m| write!(f, "Content-Length: {}\r\n\r\n{}", m.len(), m))
    }
}

impl FromStr for Response {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).map(|(json, _)| json)
    }
}

#[cfg(feature = "std")]
mod io {
    use super::*;
    use std::io::prelude::*;

    impl Response {
        /// Read a response from a reader.
        ///
        /// Returns the number of consumed bytes and the response.
        pub fn try_from_reader<R>(reader: R) -> Result<(usize, Self), Error>
        where
            R: Read,
        {
            let (n, contents) = helpers::get_content_from_reader(reader)?;
            let response = Response::parse_json(&contents)?;
            Ok((n, response))
        }

        /// Write a response to a writer and return the number of bytes written.
        pub fn try_to_writer<W>(&self, mut writer: W) -> Result<usize, Error>
        where
            W: Write,
        {
            writer
                .write(self.to_string().as_bytes())
                .map_err(|e| Error {
                    code: Error::PARSE_ERROR,
                    message: e.to_string(),
                    data: serde_json::to_value(&self).ok(),
                })
        }
    }
}
