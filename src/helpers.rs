use super::Error;
use alloc::string::ToString;
use serde_json::Value;

pub fn get_content_length(mut s: &str) -> Result<(usize, &str), Error> {
    let length;
    loop {
        let (line, rest) = s.split_once('\n').ok_or_else(|| Error {
            code: Error::INVALID_REQUEST,
            message: "the provided request header is invalid".to_string(),
            data: Some(Value::String(s.to_string())),
        })?;
        s = rest;
        let (key, value) = line.split_once(':').ok_or_else(|| Error {
            code: Error::INVALID_REQUEST,
            message: "the provided request header is invalid".to_string(),
            data: Some(Value::String(s.to_string())),
        })?;

        if key.trim().to_lowercase() == "content-length" {
            length = value.trim().parse::<usize>().map_err(|_| Error {
                code: Error::INVALID_REQUEST,
                message: "the provided request header is invalid".to_string(),
                data: Some(Value::String(s.to_string())),
            })?;
            break;
        }
    }

    loop {
        let (line, rest) = s.split_once('\n').ok_or_else(|| Error {
            code: Error::INVALID_REQUEST,
            message: "the provided request header is invalid".to_string(),
            data: Some(Value::String(s.to_string())),
        })?;
        s = rest;
        if line.trim().is_empty() {
            break;
        }
    }

    Ok((length, s))
}
