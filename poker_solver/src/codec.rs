use bytes::{Buf, BufMut, BytesMut};
use serde::Serialize;
use std::error::Error;
use std::fmt;
use std::str;
use tokio_util::codec::{Decoder, Encoder};

use crate::event::PokerEvent;

#[derive(Debug)]
pub enum PokerCodecError {
    InvalidJson,
    InvalidEvent,
    Serialize,
    Io(std::io::Error),
    Utf8(std::str::Utf8Error),
}

impl fmt::Display for PokerCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PokerCodecError::InvalidEvent => write!(f, "Unable to parse json as poker event"),
            PokerCodecError::InvalidJson => write!(f, "Invalid Json"),
            PokerCodecError::Serialize => write!(f, "Could not serialize event into json"),
            PokerCodecError::Io(e) => write!(f, "{}", e),
            PokerCodecError::Utf8(e) => write!(f, "{}", e),
        }
    }
}

impl From<std::io::Error> for PokerCodecError {
    fn from(e: std::io::Error) -> PokerCodecError {
        PokerCodecError::Io(e)
    }
}

impl From<std::str::Utf8Error> for PokerCodecError {
    fn from(e: std::str::Utf8Error) -> PokerCodecError {
        PokerCodecError::Utf8(e)
    }
}

impl Error for PokerCodecError {}

/// A poker codec using JSON to serialize messages
pub struct PokerCodec {
    next_index: usize,
    max_length: usize,
    open_brace_count: usize,
    is_discarding: bool,
}

impl PokerCodec {
    pub fn new() -> Self {
        PokerCodec {
            next_index: 0,
            open_brace_count: 0,
            max_length: usize::MAX,
            is_discarding: false,
        }
    }
}

impl Decoder for PokerCodec {
    type Item = PokerEvent;
    type Error = PokerCodecError;
    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            // deterine how far to read to
            let read_to = std::cmp::min(self.max_length.saturating_add(1), buf.len());

            self.open_brace_count = 1;
            // index of the final closing brace in the json
            let mut json_offset = None;
            for i in self.next_index + 1..read_to {
                let b: u8 = buf[i];
                if b == '{' as u8 {
                    self.open_brace_count += 1;
                }
                if b == '}' as u8 {
                    self.open_brace_count -= 1;
                }
                if self.open_brace_count == 0 {
                    json_offset = Some(i);
                    break;
                }
            }

            match (self.is_discarding, json_offset) {
                (true, Some(offset)) => {
                    buf.advance(offset + self.next_index + 1);
                    self.is_discarding = false;
                    self.next_index = 0;
                }
                (true, None) => {
                    buf.advance(read_to);
                    self.next_index = 0;
                    if buf.is_empty() {
                        return Err(PokerCodecError::InvalidJson);
                    }
                }
                (false, Some(offset)) => {
                    let json_index = offset + self.next_index;
                    self.next_index = 0;
                    let line = buf.split_to(json_index + 1);
                    let line = str::from_utf8(&line)?;
                    println!("{}", line);
                    return match serde_json::from_str(&line) {
                        Ok(event) => Ok(Some(event)),
                        Err(_) => Err(PokerCodecError::InvalidEvent),
                    };
                }
                (false, None) if buf.len() > self.max_length => {
                    self.is_discarding = true;
                    return Err(PokerCodecError::InvalidJson);
                }
                (false, None) => {
                    self.next_index = read_to;
                    return Ok(None);
                }
            }
        }
    }
}

impl<T> Encoder<T> for PokerCodec
where
    T: Serialize,
{
    type Error = PokerCodecError;
    fn encode(&mut self, event: T, buf: &mut BytesMut) -> Result<(), Self::Error> {
        match serde_json::to_string(&event) {
            Ok(event_str) => {
                buf.reserve(event_str.len() + 1);
                buf.put(event_str.as_bytes());
                Ok(())
            }
            Err(_) => Err(PokerCodecError::Serialize),
        }
    }
}

impl Default for PokerCodec {
    fn default() -> Self {
        Self::new()
    }
}
