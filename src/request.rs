use super::{helpers, Error};
use alloc::string::{String, ToString};
use core::{fmt, str::FromStr};
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

    /// Parse a message into the request, returning the remainder string
    pub fn parse(s: &str) -> Result<(Self, &str), Error> {
        let (message, remainder) = helpers::get_content_length(s)?;
        let request = Request::parse_json(message)?;
        Ok((request, remainder))
    }

    /// Parse a request from the provided JSON
    pub fn parse_json(json: &str) -> Result<Self, Error> {
        serde_json::from_str(json).map_err(|e| Error {
            code: Error::INVALID_REQUEST,
            message: e.to_string(),
            data: Some(Value::String(json.to_string())),
        })
    }
}

impl FromStr for Request {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).map(|(json, _)| json)
    }
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        serde_json::to_string(&self)
            .map_err(|_| fmt::Error)
            .and_then(|m| write!(f, "Content-Length: {}\r\n\r\n{}", m.len(), m))
    }
}

#[cfg(feature = "std")]
mod io {
    use super::*;
    use std::io::prelude::*;

    impl Request {
        /// Read a request from a reader.
        ///
        /// Returns the number of consumed bytes and the request.
        pub fn try_from_reader<R>(reader: R) -> Result<(usize, Self), Error>
        where
            R: Read,
        {
            let (n, contents) = helpers::get_content_from_reader(reader)?;
            let request = Request::parse_json(&contents)?;
            Ok((n, request))
        }

        /// Write a request to a writer and return the number of bytes written.
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
