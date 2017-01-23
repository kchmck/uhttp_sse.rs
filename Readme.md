# uhttp_sse -- HTTP Server-Sent Events protocol

[Documentation](https://docs.rs/uhttp_sse)

This crate provides a zero-copy, zero-allocation implementation of the [Server-Sent
Events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events)
(SSE) protocol for streaming events from an HTTP server.

The events can be written directly to a `TcpStream` or any other object that
implements `Write`.

## Example

```rust
use uhttp_sse::SseMessage;
use std::io::Write;

let mut buf = [0; 31];

{
    let mut sse = SseMessage::new(&mut buf[..]);
    write!(sse.event().unwrap(), "ping").unwrap();
    write!(sse.data().unwrap(), "abc").unwrap();
    write!(sse.data().unwrap(), "{}", 1337).unwrap();
}

// This would result in the "ping" event listener being triggered with the data
// payload "abc1337".
assert_eq!(&buf[..], b"event:ping\ndata:abc\ndata:1337\n\n");

```

## Usage

This [crate](https://crates.io/crates/uhttp_sse) can be used through cargo by adding
it as a dependency in `Cargo.toml`:

```toml
[dependencies]
uhttp_sse = "0.5.0"
```
and importing it in the crate root:

```rust
extern crate uhttp_sse;
```
