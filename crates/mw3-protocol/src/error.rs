use core::fmt;

/// Failure while encoding or decoding a protocol message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    /// Buffer is shorter than the minimum required size.
    UnexpectedEof { need: usize, got: usize },
    /// First four bytes do not match the expected magic value.
    BadMagic { expected: u32, got: u32 },
    /// `NumberOfEntries` or payload length does not match the buffer length.
    ResponseTruncated {
        number_of_entries: u32,
        need: usize,
        got: usize,
    },
    /// Entry count exceeds the safe decode/encode limit.
    TooManyEntries { max: usize, got: usize },
    /// Size arithmetic overflowed (response length).
    SizeOverflow,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolError::UnexpectedEof { need, got } => {
                write!(f, "unexpected eof: need {need} bytes, got {got}")
            }
            ProtocolError::BadMagic { expected, got } => {
                write!(f, "bad magic: expected 0x{expected:08X}, got 0x{got:08X}")
            }
            ProtocolError::ResponseTruncated {
                number_of_entries,
                need,
                got,
            } => write!(
                f,
                "truncated client response: {number_of_entries} entries need {need} bytes, got {got}"
            ),
            ProtocolError::TooManyEntries { max, got } => {
                write!(f, "too many entries: max {max}, got {got}")
            }
            ProtocolError::SizeOverflow => write!(f, "response size overflow"),
        }
    }
}

impl std::error::Error for ProtocolError {}
