use super::{helpers, Error};
use alloc::string::{String, ToString};
use core::{fmt, str::FromStr};
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
        let (message, remainder) = helpers::get_content_length(s)?;
        let notification = Notification::parse_json(message)?;
        Ok((notification, remainder))
    }

    /// Parse a notification from the provided JSON
    pub fn parse_json(json: &str) -> Result<Self, Error> {
        serde_json::from_str(json).map_err(|e| Error {
            code: Error::INVALID_REQUEST,
            message: e.to_string(),
            data: Some(Value::String(json.to_string())),
        })
    }
}

impl FromStr for Notification {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).map(|(json, _)| json)
    }
}

impl fmt::Display for Notification {
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

    impl Notification {
        /// Read a notification from a reader.
        ///
        /// Returns the number of consumed bytes and the notification.
        pub fn try_from_reader<R>(reader: R) -> Result<(usize, Self), Error>
        where
            R: Read,
        {
            let (n, contents) = helpers::get_content_from_reader(reader)?;
            let notification = Notification::parse_json(&contents)?;
            Ok((n, notification))
        }

        /// Write a notification to a writer and return the number of bytes written.
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
