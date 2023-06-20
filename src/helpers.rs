use super::Error;
use alloc::string::ToString;
use serde_json::Value;

/// Read the content length from the argument, returning the parsed value and remainder string.
pub fn get_content_length(mut s: &str) -> Result<(&str, &str), Error> {
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

    if s.len() < length {
        return Err(Error {
            code: Error::INVALID_REQUEST,
            message: "the provided request is invalid".to_string(),
            data: Some(Value::String(s.to_string())),
        });
    }

    Ok(s.split_at(length))
}

#[test]
fn test_get_content_length() {
    let bytes = "Foo: HTTP/1.1\r\nContent-Length: 5\r\n\r\nHelloEXTRA";
    let (message, remainder) = get_content_length(bytes).unwrap();
    assert_eq!(message.as_bytes(), b"Hello");
    assert_eq!(remainder.as_bytes(), b"EXTRA");
}

#[cfg(feature = "std")]
pub use io::get_content_from_reader;

#[cfg(feature = "std")]
mod io {
    use super::*;
    use std::io::{self, prelude::*};

    /// Read the contents length of the argument and fill a buffer with its size.
    ///
    /// Return the amount of read bytes, and the extracted bytes buffer.
    pub fn get_content_from_reader<R>(mut reader: R) -> Result<(usize, String), Error>
    where
        R: Read,
    {
        let mut n = 0;
        let length;
        loop {
            let line = reader
                .by_ref()
                .bytes()
                .take_while(|b| match b {
                    Ok(b) => *b != b'\n',
                    Err(_) => true,
                })
                .collect::<io::Result<Vec<u8>>>()
                .map_err(|e| Error {
                    code: Error::INVALID_REQUEST,
                    message: e.to_string(),
                    data: None,
                })?;
            n += line.len() + 1;

            let line = String::from_utf8(line).map_err(|e| Error {
                code: Error::INVALID_REQUEST,
                message: e.to_string(),
                data: None,
            })?;
            let (key, value) = line.split_once(':').ok_or_else(|| Error {
                code: Error::INVALID_REQUEST,
                message: "the provided request header is invalid".to_string(),
                data: Some(Value::String(line.to_string())),
            })?;
            if key.trim().to_lowercase() == "content-length" {
                length = value.trim().parse::<usize>().map_err(|_| Error {
                    code: Error::INVALID_REQUEST,
                    message: "the provided request header is invalid".to_string(),
                    data: Some(Value::String(value.to_string())),
                })?;
                break;
            }
        }

        loop {
            let line = reader
                .by_ref()
                .bytes()
                .take_while(|b| match b {
                    Ok(b) => *b != b'\n',
                    Err(_) => true,
                })
                .collect::<io::Result<Vec<u8>>>()
                .map_err(|e| Error {
                    code: Error::INVALID_REQUEST,
                    message: e.to_string(),
                    data: None,
                })?;
            n += line.len() + 1;

            let line = String::from_utf8(line).map_err(|e| Error {
                code: Error::INVALID_REQUEST,
                message: e.to_string(),
                data: None,
            })?;

            if line.trim().is_empty() {
                break;
            }
        }

        let mut buffer = vec![0u8; length];
        n += buffer.len();
        reader.read_exact(&mut buffer).map_err(|e| Error {
            code: Error::INVALID_REQUEST,
            message: e.to_string(),
            data: None,
        })?;

        let contents = String::from_utf8(buffer).map_err(|e| Error {
            code: Error::PARSE_ERROR,
            message: e.to_string(),
            data: None,
        })?;

        Ok((n, contents))
    }

    #[test]
    fn test_get_buffer_from_reader() {
        let bytes = "Foo: HTTP/1.1\r\nContent-Length: 5\r\n\r\nHelloEXTRA";
        let (n, contents) = get_content_from_reader(bytes.as_bytes()).unwrap();
        assert_eq!(n, 41);
        assert_eq!(contents.as_bytes(), b"Hello");
    }
}
