use super::{helpers, Error, Notification, Request, Response};
use alloc::string::ToString;
use core::str::FromStr;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Request(Request),
    Notification(Notification),
    Response(Response),
}

impl Message {
    pub fn parse(s: &str) -> Result<(Self, &str), Error> {
        let (message, _) = helpers::get_content_length(s)?;
        let (message, _) = s.split_at(message);
        let message: Value = serde_json::from_str(message).map_err(|e| Error {
            code: Error::INVALID_REQUEST,
            message: e.to_string(),
            data: Some(Value::String(s.to_string())),
        })?;
        if message.get("method").is_some() && message.get("id").is_some() {
            Request::parse(s).map(|(r, s)| (Self::Request(r), s))
        } else if message.get("method").is_some() {
            Notification::parse(s).map(|(n, s)| (Self::Notification(n), s))
        } else {
            Response::parse(s).map(|(r, s)| (Self::Response(r), s))
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
