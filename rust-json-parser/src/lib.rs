//! # Rust JSON Parser
//!
//! A JSON parser built for the Rust cohort, with optional Python bindings.
//!
//! Use [`JsonParser`] to tokenize input and parse it into a [`JsonValue`].
//! Accessors on the resulting value let you inspect nested data without
//! matching every enum variant yourself. Failures are reported as [`JsonError`].
//!
//! # Quick start
//!
//! ```rust
//! use rust_json_parser::{JsonParser, JsonValue};
//!
//! let value = JsonParser::new(r#"{"name": "Alice", "active": true}"#)?.parse()?;
//! assert_eq!(value.get("name").and_then(JsonValue::as_str), Some("Alice"));
//! assert_eq!(value.get("active").and_then(JsonValue::as_bool), Some(true));
//! # Ok::<(), rust_json_parser::JsonError>(())
//! ```
//!
//! # Formatting
//!
//! [`ToString::to_string`] produces compact output, while
//! [`JsonValue::pretty_print`] adds line breaks and indentation. Object key
//! order is unspecified because objects use a [`std::collections::HashMap`].
//!
//! # Python support
//!
//! The default `python` feature builds the PyO3 bindings for the Python package's
//! `parse_json`, `parse_json_file`, and `dumps` functions. These functions are
//! Python entry points; Rust callers use [`JsonParser`]. Disable default features
//! to build or test the Rust library without Python.
//!
//! # Current limitations
//!
//! Numbers are stored as [`f64`], so large integers may lose precision. The
//! tokenizer does not support exponent notation or Unicode surrogate-pair
//! escapes, and its validation is not fully strict (for example, it accepts
//! leading zeroes in numbers). Parsing and formatting recurse without a depth
//! limit, so deeply nested input can exhaust the stack. Error positions use
//! different units depending on the parsing stage; see [`JsonError`].

mod error;
mod parser;
mod tokenizer;
mod value;

// Keep the module layout private while exposing the supported Rust API here.
pub use error::JsonError;
pub use parser::JsonParser;
pub use tokenizer::Token;
pub use value::JsonValue;

/// A result whose error type is [`JsonError`].
///
/// # Examples
///
/// ```
/// use rust_json_parser::{JsonParser, JsonValue, Result};
///
/// fn read_value(input: &str) -> Result<JsonValue> {
///     JsonParser::new(input)?.parse()
/// }
///
/// assert_eq!(read_value("null")?, JsonValue::Null);
/// # Ok::<(), rust_json_parser::JsonError>(())
/// ```
pub type Result<T> = std::result::Result<T, JsonError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration() -> Result<()> {
        // Test the full parsing pipeline
        let value = JsonParser::new("42.0")?.parse()?;
        assert_eq!(value, JsonValue::Number(42.0));

        let value = JsonParser::new("true")?.parse()?;
        assert_eq!(value, JsonValue::Boolean(true));

        let value = JsonParser::new("null")?.parse()?;
        assert_eq!(value, JsonValue::Null);

        let value = JsonParser::new(r#""hello""#)?.parse()?;
        assert_eq!(value, JsonValue::String("hello".to_string()));
        Ok(())
    }

    #[test]
    fn test_error_propagation() -> Result<()> {
        // Test that errors propagate properly with correct details
        let result = JsonParser::new("@invalid@");
        assert!(result.is_err());

        // Validate error details through pattern matching
        match result {
            Err(JsonError::UnexpectedToken {
                expected,
                found,
                position,
            }) => {
                assert_eq!(expected, "valid JSON token");
                assert_eq!(found, "@");
                assert_eq!(position, 0);
                Ok(())
            }
            _ => panic!("Expected UnexpectedToken error"),
        }
    }
}

#[cfg(feature = "python")]
mod python_bindings;
