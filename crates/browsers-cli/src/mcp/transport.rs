//! Newline-delimited frame reader.
//!
//! A frame is bounded by MAX_FRAME_BYTES so a hostile or broken client cannot
//! grow the heap without bound. An over-long frame is reported once and its
//! remainder is discarded up to the next newline, so the stream resynchronises
//! instead of executing the leftover bytes as a fresh request.

use std::io::{self, BufRead};

/// MAX_FRAME_BYTES caps a single JSON-RPC frame. It matches the cap the other
/// headless MCP servers use.
pub const MAX_FRAME_BYTES: usize = 8 << 20;

/// Frame is one read from the transport.
#[derive(Debug)]
pub enum Frame {
    /// Message is a complete frame, still carrying its trailing newline. The
    /// server trims surrounding whitespace before dispatch.
    Message(String),
    /// TooLong means the frame exceeded MAX_FRAME_BYTES and was discarded.
    TooLong,
    /// Eof is a clean end of stream, which is how a client signals shutdown.
    Eof,
    /// Error is an underlying read failure.
    Error(io::Error),
}

/// FrameReader turns a byte stream into bounded frames.
pub struct FrameReader<'a, R: BufRead> {
    reader: &'a mut R,
}

impl<'a, R: BufRead> FrameReader<'a, R> {
    /// new wraps a buffered reader.
    pub fn new(reader: &'a mut R) -> Self {
        FrameReader { reader }
    }

    /// next_frame reads the next frame, discarding the tail of an over-long one.
    pub fn next_frame(&mut self) -> Frame {
        let mut frame: Vec<u8> = Vec::new();
        let mut overflowed = false;

        loop {
            let available = match self.reader.fill_buf() {
                Ok(buffer) => buffer,
                Err(err) => return Frame::Error(err),
            };
            if available.is_empty() {
                // End of stream. Any bytes already read form a final frame: a
                // client whose last frame lacks a trailing newline is normal,
                // so this must not be reported as a parse error.
                return finish(frame, overflowed, Frame::Eof);
            }

            match available.iter().position(|byte| *byte == b'\n') {
                Some(index) => {
                    append(&mut frame, &available[..=index], &mut overflowed);
                    let consumed = index + 1;
                    self.reader.consume(consumed);
                    if overflowed {
                        return Frame::TooLong;
                    }
                    return match String::from_utf8(frame) {
                        Ok(text) => Frame::Message(text),
                        // Invalid UTF-8 cannot be JSON, so report an empty
                        // frame; the server answers with a parse error.
                        Err(_) => Frame::Message(String::new()),
                    };
                }
                None => {
                    let len = available.len();
                    append(&mut frame, available, &mut overflowed);
                    self.reader.consume(len);
                }
            }
        }
    }
}

impl PartialEq for Frame {
    /// Compares by variant and payload, treating two read errors as equal so a
    /// test can assert on the kind of failure rather than its identity.
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Frame::Message(a), Frame::Message(b)) => a == b,
            (Frame::TooLong, Frame::TooLong) | (Frame::Eof, Frame::Eof) => true,
            (Frame::Error(a), Frame::Error(b)) => a.kind() == b.kind(),
            _ => false,
        }
    }
}

/// append copies a chunk into the frame, stopping once the cap is exceeded. The
/// bytes are still consumed from the reader so the tail is discarded rather
/// than parsed as a new frame.
fn append(frame: &mut Vec<u8>, chunk: &[u8], overflowed: &mut bool) {
    if *overflowed {
        return;
    }
    if frame.len() + chunk.len() > MAX_FRAME_BYTES {
        *overflowed = true;
        frame.clear();
        frame.shrink_to_fit();
        return;
    }
    frame.extend_from_slice(chunk);
}

/// finish turns accumulated state into a frame at end of stream.
fn finish(frame: Vec<u8>, overflowed: bool, eof: Frame) -> Frame {
    if overflowed {
        return Frame::TooLong;
    }
    match eof {
        Frame::Eof if frame.is_empty() => Frame::Eof,
        Frame::Eof => match String::from_utf8(frame) {
            Ok(text) => Frame::Message(text),
            Err(_) => Frame::Message(String::new()),
        },
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    fn frames(input: &str) -> Vec<Frame> {
        let mut reader = BufReader::new(input.as_bytes());
        let mut frame_reader = FrameReader::new(&mut reader);
        let mut collected = Vec::new();
        loop {
            match frame_reader.next_frame() {
                Frame::Eof => return collected,
                other => collected.push(other),
            }
        }
    }

    #[test]
    fn reads_two_frames_then_eof() {
        let got = frames("{\"a\":1}\n{\"b\":2}\n");
        assert_eq!(
            got,
            vec![
                Frame::Message("{\"a\":1}\n".to_string()),
                Frame::Message("{\"b\":2}\n".to_string()),
            ]
        );
    }

    #[test]
    fn a_final_line_without_a_newline_is_still_a_frame() {
        let got = frames("{\"a\":1}");
        assert_eq!(got, vec![Frame::Message("{\"a\":1}".to_string())]);
    }

    #[test]
    fn an_oversized_frame_is_reported_and_its_tail_discarded() {
        let padding = "x".repeat(MAX_FRAME_BYTES + 5_000);
        let smuggled = r#"{"jsonrpc":"2.0","id":99,"method":"ping"}"#;
        let got = frames(&format!(
            "{padding}{smuggled}\n{{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"ping\"}}\n"
        ));

        assert_eq!(
            got[0],
            Frame::TooLong,
            "the oversized frame must be refused"
        );
        assert_eq!(
            got[1],
            Frame::Message("{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"ping\"}\n".to_string()),
            "the stream must resynchronise on the next newline, not run the tail"
        );
        assert_eq!(
            got.len(),
            2,
            "the tail of the oversized frame must not be parsed"
        );
    }

    #[test]
    fn a_frame_at_the_cap_is_accepted() {
        // Exactly at the cap: one byte of payload plus the newline.
        let payload = "y".repeat(MAX_FRAME_BYTES - 1);
        let got = frames(&format!("{payload}\n"));
        assert_eq!(got.len(), 1);
        assert!(matches!(got[0], Frame::Message(_)));
    }
}
