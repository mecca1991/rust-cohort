//! Error types shared by tokenization, parsing, and the Python bindings.
//!
//! [`JsonError`] implements [`std::fmt::Display`] for readable messages and
//! [`std::error::Error`] for use with standard Rust error handling.

use std::fmt;

/// A tokenization, parsing, or file-access failure.
///
/// Positions are zero-based, but are not consistently source offsets:
/// tokenization uses character indices (not UTF-8 byte offsets), while parsing
/// uses token indices. Some tokenizer errors report a fixed position of zero;
/// others report the cursor after consuming the offending text. Do not use
/// these positions directly to slice the original input.
///
/// # Examples
///
/// ```
/// use rust_json_parser::{JsonError, JsonParser};
///
/// match JsonParser::new("@") {
///     Err(JsonError::UnexpectedToken { found, position, .. }) => {
///         assert_eq!(found, "@");
///         assert_eq!(position, 0);
///     }
///     _ => panic!("expected an invalid-token error"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum JsonError {
    /// The input contains an unrecognized character, keyword, or misplaced token.
    UnexpectedToken {
        /// A description of the expected token or value.
        expected: String,
        /// The unexpected input or a debug representation of the token.
        found: String,
        /// The reported character or token index; see the enum's position notes.
        position: usize,
    },
    /// The input ends before a required value, delimiter, or closing quote.
    UnexpectedEndOfInput {
        /// A description of what was needed to continue parsing.
        expected: String,
        /// The reported character or token index; see the enum's position notes.
        position: usize,
    },
    /// A numeric candidate cannot be converted to an `f64`.
    InvalidNumber {
        /// Details of the failed conversion, currently the float parser's error.
        value: String,
        /// The character cursor after scanning the numeric candidate.
        position: usize,
    },
    /// A backslash in a string is followed by an unsupported escape character.
    InvalidEscape {
        /// The character immediately following the backslash.
        char: char,
        /// The character index of the unsupported escape character.
        position: usize,
    },
    /// A `\uXXXX` escape has invalid hex digits or is not a Unicode scalar value.
    ///
    /// Surrogate escapes are rejected, including those in surrogate pairs.
    InvalidUnicode {
        /// The invalid character or the hex digits collected so far.
        sequence: String,
        /// The character cursor where validation failed, possibly after the digits.
        position: usize,
    },
    /// A file operation failed.
    ///
    /// The Rust parser does not perform file I/O. This variant is available to
    /// callers and maps to Python's `OSError` when converted through PyO3.
    IoError {
        /// A description of the I/O failure.
        message: String,
    },
    /// A requested file could not be found.
    ///
    /// Like [`JsonError::IoError`], this variant is not emitted by the Rust parser.
    FileNotFound {
        /// The path of the missing file.
        path: String,
    },
}

/// Formats an error with its context and, when available, reported position.
impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonError::UnexpectedToken {
                expected,
                found,
                position,
            } => {
                write!(
                    f,
                    "Invalid Token provided! Expected {} but found {} at position {}",
                    expected, found, position
                )
            }
            JsonError::UnexpectedEndOfInput { expected, position } => {
                write!(
                    f,
                    "Incorrect End of Input! Expected {} at position {}",
                    expected, position
                )
            }
            JsonError::InvalidNumber { value, position } => {
                write!(f, "Invalid number {} at position {}", value, position)
            }
            JsonError::InvalidEscape { char, position } => {
                write!(
                    f,
                    "Invalid escape character {} at position {}",
                    char, position
                )
            }
            JsonError::InvalidUnicode { sequence, position } => {
                write!(f, "Invalid unicode {} at position {}", sequence, position)
            }
            JsonError::IoError { message } => {
                write!(f, "IO Error {}", message)
            }
            JsonError::FileNotFound { path } => {
                write!(f, "File not found at {}", path)
            }
        }
    }
}

impl std::error::Error for JsonError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = JsonError::UnexpectedToken {
            expected: "number".to_string(),
            found: "@".to_string(),
            position: 5,
        };

        // Error should be Debug-printable
        assert!(format!("{:?}", error).contains("UnexpectedToken"));
    }

    #[test]
    fn test_error_display() {
        let error = JsonError::UnexpectedToken {
            expected: "valid JSON".to_string(),
            found: "@".to_string(),
            position: 0,
        };

        let message = format!("{}", error);
        assert!(message.contains("position 0"));
        assert!(message.contains("valid JSON"));
        assert!(message.contains("@"));
    }

    #[test]
    fn test_error_variants() {
        let token_error = JsonError::UnexpectedToken {
            expected: "number".to_string(),
            found: "x".to_string(),
            position: 3,
        };

        let eof_error = JsonError::UnexpectedEndOfInput {
            expected: "closing quote".to_string(),
            position: 10,
        };

        let num_error = JsonError::InvalidNumber {
            value: "12.34.56".to_string(),
            position: 0,
        };

        // All variants should be Debug-printable
        assert!(!format!("{:?}", token_error).is_empty());
        assert!(!format!("{:?}", eof_error).is_empty());
        assert!(!format!("{:?}", num_error).is_empty());
    }

    #[test]
    fn test_invalid_escape_display() {
        let err = JsonError::InvalidEscape {
            char: 'q',
            position: 5,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("escape"));
        assert!(msg.contains("q"));
    }

    #[test]
    fn test_invalid_unicode_display() {
        let err = JsonError::InvalidUnicode {
            sequence: "00GG".to_string(),
            position: 3,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("unicode") || msg.contains("Unicode"));
    }

    #[test]
    fn test_error_is_std_error() {
        let err = JsonError::InvalidEscape {
            char: 'x',
            position: 0,
        };
        let _: &dyn std::error::Error = &err; // Must implement Error trait
    }
}
