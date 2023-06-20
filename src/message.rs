use super::{helpers, Error, Notification, Request, Response};
use alloc::string::ToString;
use core::{fmt, str::FromStr};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC message
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    /// JSON-RPC request
    Request(Request),
    /// JSON-RPC notification
    Notification(Notification),
    /// JSON-RPC response
    Response(Response),
}

impl Message {
    /// Parse a string into the message, returning the message and the remaining string
    pub fn parse(s: &str) -> Result<(Self, &str), Error> {
        let (message, remainder) = helpers::get_content_length(s)?;
        let message = Message::parse_json(message)?;
        Ok((message, remainder))
    }

    /// Parse a message from the provided JSON
    pub fn parse_json(json: &str) -> Result<Self, Error> {
        let value: Value = serde_json::from_str(json).map_err(|e| Error {
            code: Error::INVALID_REQUEST,
            message: e.to_string(),
            data: Some(Value::String(json.to_string())),
        })?;
        if value.get("method").is_some() && value.get("id").is_some() {
            Request::parse_json(json).map(Self::Request)
        } else if value.get("method").is_some() {
            Notification::parse_json(json).map(Self::Notification)
        } else {
            Response::parse_json(json).map(Self::Response)
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Message::Request(r) => r.fmt(f),
            Message::Notification(n) => n.fmt(f),
            Message::Response(r) => r.fmt(f),
        }
    }
}

impl FromStr for Message {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).map(|(json, _)| json)
    }
}

impl From<Request> for Message {
    fn from(request: Request) -> Self {
        Self::Request(request)
    }
}

impl From<Notification> for Message {
    fn from(notification: Notification) -> Self {
        Self::Notification(notification)
    }
}

impl From<Response> for Message {
    fn from(response: Response) -> Self {
        Self::Response(response)
    }
}

impl TryFrom<Message> for Request {
    type Error = Error;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        match value {
            Message::Request(request) => Ok(request),
            _ => Err(Error {
                code: Error::INVALID_REQUEST,
                message: "the provided message is not a request".to_string(),
                data: serde_json::to_value(value).ok(),
            }),
        }
    }
}

impl TryFrom<Message> for Notification {
    type Error = Error;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        match value {
            Message::Notification(notification) => Ok(notification),
            _ => Err(Error {
                code: Error::INVALID_REQUEST,
                message: "the provided message is not a notification".to_string(),
                data: serde_json::to_value(value).ok(),
            }),
        }
    }
}

impl TryFrom<Message> for Response {
    type Error = Error;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        match value {
            Message::Response(response) => Ok(response),
            _ => Err(Error {
                code: Error::INVALID_REQUEST,
                message: "the provided message is not a response".to_string(),
                data: serde_json::to_value(value).ok(),
            }),
        }
    }
}

#[cfg(feature = "std")]
mod io {
    use super::*;
    use std::io::prelude::*;

    impl Message {
        /// Read a message from a reader.
        ///
        /// Returns the number of consumed bytes and the message.
        pub fn try_from_reader<R>(reader: R) -> Result<(usize, Self), Error>
        where
            R: Read,
        {
            let (n, contents) = helpers::get_content_from_reader(reader)?;
            let message = Message::parse_json(&contents)?;
            Ok((n, message))
        }

        /// Write a message to a writer and return the number of bytes written.
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

    #[test]
    fn test_try_from_reader() {
        let input = r#"Content-Length: 75

{"jsonrpc":"2.0","id":"3162690c-fe69-4b78-b418-0b2e8326ac08","result":true}"#;

        let (consumed, _message) = Message::try_from_reader(input.as_bytes()).unwrap();
        assert_eq!(consumed, input.len());
    }
}
