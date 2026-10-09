// Week 2: JSON Parser with Error Handling

// Declare modules - tell Rust which files contain your code
// `mod error;` looks for src/error.rs
mod error;
mod parser;
mod tokenizer;
mod value;

// Re-export types - make them accessible from the top level
// Without this: users write `use my_lib::parser::parse_json`
// With this: users write `use my_lib::parse_json` (cleaner!)
pub use error::JsonError;
pub use parser::JsonParser;
pub use tokenizer::Token;
pub use value::JsonValue;

// Type alias for convenience
// Users can write Result<JsonValue> instead of std::result::Result<JsonValue, JsonError>
pub type Result<T> = std::result::Result<T, JsonError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration() -> Result<()> {
        // Test the full parsing pipeline
        let mut parser = JsonParser::new("42.0")?;
        assert_eq!(parser.parse()?, JsonValue::Number(42.0));

        let mut parser = JsonParser::new("true")?;
        assert_eq!(parser.parse()?, JsonValue::Boolean(true));

        let mut parser = JsonParser::new("null")?;
        assert_eq!(parser.parse()?, JsonValue::Null);

        let mut parser = JsonParser::new(r#""hello""#)?;
        assert_eq!(parser.parse()?, JsonValue::String("hello".to_string()));
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
