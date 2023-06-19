# jsonrpc-base

A minimalistic types implementation of the JSON-RPC protocol.

[![crates.io](https://img.shields.io/crates/v/jsonrpc-base?label=latest)](https://crates.io/crates/jsonrpc-base)
[![Documentation](https://docs.rs/jsonrpc-base/badge.svg)](https://docs.rs/jsonrpc-base/)
[![License](https://img.shields.io/crates/l/jsonrpc-base.svg)]()

## Example

```rust
use jsonrpc_base::Request;

let (id, request) = Request::new("foo/barBaz")
    .with_params(vec![1, 2, 3])
    .expect("vec JSON parse will not fail")
    .prepare();

let mut lines = request.lines();
assert_eq!(lines.next(), Some("Content-Length: 100"));
assert_eq!(lines.next(), Some(""));

let mut message = String::new();
message.push_str(r#"{"jsonrpc":"2.0","id":"#);
message.push_str(id.to_string().as_str());
message.push_str(r#","method":"foo/barBaz","params":[1,2,3]}"#);
assert_eq!(lines.next(), Some(message.as_str()));
```
