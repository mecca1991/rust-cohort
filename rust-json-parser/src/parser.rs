use crate::error::JsonError;
use crate::tokenizer::{Token, Tokenizer};
use crate::value::JsonValue;

type Result<T> = std::result::Result<T, JsonError>;

pub struct JsonParser {
    tokens: Vec<Token>,
    position: usize,
}

impl JsonParser {
    pub fn new(input: &str) -> Result<JsonParser> {
        let mut tokenizer = Tokenizer::new(input.trim());
        let tokens = tokenizer.tokenize();

        match tokens {
            Ok(tokens) => Ok(Self {
                tokens,
                position: 0,
            }),
            Err(_) => {
                Err(JsonError::UnexpectedEndOfInput {
                    expected: "JSON value".to_string(),
                    position: 0,
                })
            }
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue> {
        match self.tokens.as_slice() {
            [] => Err(JsonError::UnexpectedEndOfInput {
                expected: "JSON value".to_string(),
                position: 0,
            }),
            [Token::Boolean(value)] => Ok(JsonValue::Boolean(*value)),
            [Token::String(value)] => Ok(JsonValue::String(value.clone())),
            [Token::Number(value)] => Ok(JsonValue::Number(*value)),
            [Token::Null] => Ok(JsonValue::Null),
            [token] => Err(JsonError::UnexpectedToken {
                expected: "JSON value".to_string(),
                found: format!("{:?}", token),
                position: 0,
            }),
            [_, second, ..] => Err(JsonError::UnexpectedToken {
                expected: "End of input".to_string(),
                found: format!("{second:?}"),
                position: 0,
            }),
        }
    }

    pub fn advance(&mut self) -> Option<&Token> {
        self.position += 1;
        self.tokens.get(self.position)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    type Result<T> = std::result::Result<T, JsonError>;

    #[test]
    fn test_parse_string() -> Result<()> {
        let mut parser = JsonParser::new(r#""hello world""#)?;
        let result = parser.parse()?;
        assert_eq!(result, JsonValue::String("hello world".to_string()));
        Ok(())
    }

    #[test]
    fn test_parse_empty_string() -> Result<()> {
        let mut parser = JsonParser::new(r#""""#)?;
        let result = parser.parse()?;
        assert_eq!(result, JsonValue::String(String::new()));
        Ok(())
    }
    #[test]
    fn test_parse_number() -> Result<()> {
        let mut parser = JsonParser::new("42")?;
        let value = parser.parse()?;
        assert_eq!(value, JsonValue::Number(42.0));
        Ok(())
    }

    #[test]
    fn test_parse_boolean_true() -> Result<()> {
        let mut parser = JsonParser::new("true")?;
        let value = parser.parse()?;
        assert_eq!(value, JsonValue::Boolean(true));
        Ok(())
    }

    #[test]
    fn test_parse_null() -> Result<()> {
        let mut parser = JsonParser::new("null")?;
        let value = parser.parse()?;
        assert_eq!(value, JsonValue::Null);
        Ok(())
    }

    #[test]
    fn test_parse_simple_string() -> Result<()> {
        let mut parser = JsonParser::new(r#""hello""#)?;
        let value = parser.parse()?;
        assert_eq!(value, JsonValue::String("hello".to_string()));
        Ok(())
    }
    #[test]
    fn test_parse_error_empty() {
        let result = JsonParser::new("");

        assert!(result.is_err());

        match result {
            Err(JsonError::UnexpectedEndOfInput { expected, position }) => {
                assert_eq!(expected, "JSON value");
                assert_eq!(position, 0);
            }
            _ => panic!("Expected UnexpectedEndOfInput error"),
        }
    }

    #[test]
    fn test_parse_error_invalid_token() {
        let result = JsonParser::new("@");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_with_whitespace() -> Result<()> {
        let mut parser = JsonParser::new("  42  ")?;
        let result = parser.parse()?;
        assert_eq!(result, JsonValue::Number(42.0));

        let mut parser = JsonParser::new("\n\ttrue\n")?;
        let result = parser.parse()?;
        assert_eq!(result, JsonValue::Boolean(true));
        Ok(())
    }

    // #[test]
    // fn test_result_pattern_matching() {
    //     let mut parser = JsonParser::new("42")?;
    //     let result: JsonValue = parser.parse()?;
    //     match result {
    //         Ok(JsonValue::Number(n)) => assert_eq!(n, 42.0),
    //         _ => panic!("Expected successful number parse"),
    //     }
    //     let mut parser = JsonParser::new("@invalid@");
    //     let result = parser.parse();

    //     match result {
    //         Err(JsonError::UnexpectedToken { .. }) => {} // Expected
    //         _ => panic!("Expected UnexpectedToken error"),
    //     }
    // }
    #[test]
    fn test_parser_creation() {
        let parser = JsonParser::new("42");
        assert!(parser.is_ok());
    }

    #[test]
    fn test_parser_creation_tokenize_error() {
        let parser = JsonParser::new(r#""\q""#); // Invalid escape
        assert!(parser.is_err());
    }
    #[test]
    fn test_parse_negative_number() -> Result<()> {
        let mut parser = JsonParser::new("-3.14")?;
        let value = parser.parse()?;
        assert_eq!(value, JsonValue::Number(-3.14));
        Ok(())
    }

    #[test]
    fn test_parse_boolean_false() -> Result<()> {
        let mut parser = JsonParser::new("false")?;
        let value = parser.parse()?;
        assert_eq!(value, JsonValue::Boolean(false));
        Ok(())
    }

    #[test]
    fn test_parse_empty_input() {
        // Could fail at tokenization (no tokens) or parsing (empty token list)
        // Either is acceptable - just verify it's an error
        let result = match JsonParser::new("") {
            Ok(mut parser) => parser.parse(),
            Err(e) => Err(e),
        };
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_whitespace_only() {
        let result = match JsonParser::new("   ") {
            Ok(mut parser) => parser.parse(),
            Err(e) => Err(e),
        };
        assert!(result.is_err());
    }
}
