//! This crate provides a zero-copy, zero-allocation implementation of the [Server-Sent
//! Events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events)
//! (SSE) protocol for streaming events from an HTTP server.
//!
//! The events can be written directly to a `TcpStream` or any other object that
//! implements `Write`.
//!
//! ## Example
//!
//! ```rust
//! use uhttp_sse::SseMessage;
//! use std::io::Write;
//!
//! let mut buf = [0; 31];
//!
//! {
//!     let mut sse = SseMessage::new(&mut buf[..]);
//!     write!(sse.event().unwrap(), "ping").unwrap();
//!     write!(sse.data().unwrap(), "abc").unwrap();
//!     write!(sse.data().unwrap(), "{}", 1337).unwrap();
//! }
//!
//! // This would result in the "ping" event listener being triggered with the data
//! // payload "abc1337".
//! assert_eq!(&buf[..], b"event:ping\ndata:abc\ndata:1337\n\n");
//!
//! ```

use std::io::Write;

/// An SSE [message](https://www.w3.org/TR/2012/WD-eventsource-20120426).
///
/// Each message consists of any number of fields followed by a message terminating
/// sequence. The member functions allow "appending" a field to the message any number of
/// times, and when the `SseMessage` goes out of scope, the message is flushed and
/// terminated.
pub struct SseMessage<W: Write>(W);

impl<W: Write> SseMessage<W> {
    /// Create a new `SseMessage` to write into the given stream.
    pub fn new(stream: W) -> Self {
        SseMessage(stream)
    }

    /// Append a data field.
    ///
    /// This is the data payload passed into the browser event listener callback when the
    /// message is terminated. It can be raw text, JSON, or any other format. If a
    /// message contains multiple data fields, the browser concatenates their values until
    /// the end of the message.
    ///
    /// This field is the only "required" one, in that a message with an empty data field
    /// won't trigger any event listener in the browser.
    pub fn data(&mut self) -> std::io::Result<SseField<&mut W>> {
        SseField::new(&mut self.0, "data")
    }

    /// Append an event name field.
    ///
    /// This optional field tags the current message with an event name, which causes the
    /// browser to trigger an event listener specifically for that event.
    pub fn event(&mut self) -> std::io::Result<SseField<&mut W>> {
        SseField::new(&mut self.0, "event")
    }

    /// Append an event ID field.
    ///
    /// This optional field sets the "last event ID" of the current event stream.
    pub fn id(&mut self) -> std::io::Result<SseField<&mut W>> {
        SseField::new(&mut self.0, "id")
    }

    /// Append a retry field.
    ///
    /// This optional field must be an integer and sets the reconnection time of the
    /// current event stream, which is the millisecond delay used when the browser
    /// attempts to reestablish a connection to the stream.
    pub fn retry(&mut self) -> std::io::Result<SseField<&mut W>> {
        SseField::new(&mut self.0, "retry")
    }
}

/// Writes the message terminating sequence and flushes on drop.
impl<W: Write> Drop for SseMessage<W> {
    fn drop(&mut self) {
        self.0.write(&b"\n"[..]).is_ok();
        self.0.flush().is_ok();
    }
}

/// A field in an SSE message.
///
/// Each field has a name and a value. The assigned name is automatically written when the
/// object is initialized, and the value is then appended by writing into the `SseField`
/// object any number of times. When the `SseField` object goes out of scope, the field is
/// terminated.
///
/// The written value must not contain any `\n` newline characters.
pub struct SseField<W: Write>(W);

impl<W: Write> SseField<W> {
    /// Create a new `SseField` to write into the given stream with the given field name.
    fn new(mut stream: W, name: &'static str) -> std::io::Result<Self> {
        try!(write!(&mut stream, "{}:", name));
        Ok(SseField(stream))
    }
}

/// Appends to the value of the current field.
impl<W: Write> Write for SseField<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> { self.0.write(buf) }
    fn flush(&mut self) -> std::io::Result<()> { self.0.flush() }
}

/// Writes the field terminating sequence on drop.
impl<W: Write> Drop for SseField<W> {
    fn drop(&mut self) { self.0.write(b"\n").is_ok(); }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_sse_field() {
        let mut buf = [0u8; 37];

        {
            let mut f = SseField::new(&mut buf[..], "hello").unwrap();

            write!(f, "a message {}", 1337).unwrap();
            write!(f, " another message").unwrap();
        }

        assert_eq!(&buf[..], &b"hello:a message 1337 another message\n"[..]);
    }

    #[test]
    fn test_sse_msg() {
        let mut buf = [0u8; 44];

        {
            let mut msg = SseMessage::new(&mut buf[..]);

            write!(msg.event().unwrap(), "1337").unwrap();
            write!(msg.data().unwrap(), "abc").unwrap();
            write!(msg.data().unwrap(), "def").unwrap();
            write!(msg.id().unwrap(), "42").unwrap();
            write!(msg.retry().unwrap(), "7").unwrap();
        }

        assert_eq!(&buf[..], &b"event:1337\ndata:abc\ndata:def\nid:42\nretry:7\n\n"[..]);

        let mut buf = [0u8; 20];

        {
            let mut c = Cursor::new(&mut buf[..]);

            {
                let mut msg = SseMessage::new(&mut c);
                write!(msg.data().unwrap(), "abc").unwrap();
            }

            {
                let mut msg = SseMessage::new(&mut c);
                write!(msg.data().unwrap(), "def").unwrap();
            }
        }

        assert_eq!(&buf[..], &b"data:abc\n\ndata:def\n\n"[..]);
    }
}
