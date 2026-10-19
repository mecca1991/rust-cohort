use std::collections::HashMap;

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
        if input.is_empty() {
            return Err(JsonError::UnexpectedEndOfInput {
                expected: "JSON value".to_string(),
                position: 0,
            });
        }
        let tokens = Tokenizer::new(input).tokenize()?;
        Ok(Self {
            tokens,
            position: 0,
        })
    }

    pub fn parse(&mut self) -> Result<JsonValue> {
        self.parse_value()
    }

    pub fn parse_value(&mut self) -> Result<JsonValue> {
        let token = self.tokens.get(self.position).cloned();
        if let Some(token) = token {
            match token {
                Token::Boolean(value) => {
                    self.advance();
                    Ok(JsonValue::Boolean(value))
                },
                Token::String(value) => {
                    self.advance();
                    Ok(JsonValue::String(value))
                },
                Token::Number(value) => {
                    self.advance();
                    Ok(JsonValue::Number(value))
                },
                Token::LeftBracket => {
                    self.advance();
                    self.parse_array()
                },
                Token::LeftBrace => {
                    self.parse_object()
                },
                Token::Null => {
                    self.advance();
                    Ok(JsonValue::Null)
                },
                _ => Err(JsonError::UnexpectedEndOfInput {
                    expected: "JSON value".to_string(),
                    position: self.position,
                }),
            }
        } else {
            Err(JsonError::UnexpectedToken {
                expected: "JSON value".to_string(),
                found: format!("{:?}", token),
                position: 0,
            })
        }
    }

    fn advance(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.position).cloned();
        if self.is_at_end() {
            token
        } else {
            self.position += 1;
            token
        }
    }    

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.position).cloned()
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }

    fn parse_object(&mut self) -> Result<JsonValue> {
        let mut hash_items: HashMap<String, JsonValue> = HashMap::new();
        let mut current_key = String::new();
        while let Some(token) = self.tokens.get(self.position) {
            match token {
                Token::LeftBrace => {
                    let next_token = self.advance();
                    if next_token == Some(Token::RightBrace) {
                        continue;
                    }
                    let parsed_key = self.parse_value()?;
                    let key_slice = parsed_key.as_str();
                    if let Some(key) = key_slice {
                        current_key = key.to_string();
                        hash_items.insert(current_key.clone(), JsonValue::Null);
                        continue;
                    } else {
                        return Err(JsonError::UnexpectedEndOfInput {
                            expected: "JSON value".to_string(),
                            position: self.position,
                        });
                    }
                }
                Token::RightBrace => {
                    self.advance();
                    break;
                }
                Token::Comma => {
                    self.advance();
                    current_key = self.parse_value()?.as_str().unwrap().to_string();
                }
                Token::Colon => {
                    self.advance();
                    let value = self.parse_value()?;
                    hash_items.insert(current_key.clone(), value);
                    continue;
                }
                _ => {
                    self.advance();
                }
            }
        }
        Ok(JsonValue::Object(hash_items))
    }

    fn parse_array(&mut self) -> Result<JsonValue> {
        let mut array_data: Vec<JsonValue> = Vec::new();
        while let Some(token) = self.peek() {
            match token {
                Token::LeftBracket => {
                    self.advance();
                    array_data.push(self.parse_array()?);
                }
                Token::RightBracket => {
                    self.advance();
                    break;
                }
                Token::Comma => {
                    self.advance();
                }
                _ => {
                    array_data.push(self.parse_value()?);
                }
            }
        }
        Ok(JsonValue::Array(array_data))
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

    #[test]
    fn test_result_pattern_matching() -> Result<()> {
        let mut parser = JsonParser::new("42")?;
        let result = parser.parse();
        match result {
            Ok(JsonValue::Number(n)) => assert_eq!(n, 42.0),
            _ => panic!("Expected successful number parse"),
        }

        let result = JsonParser::new("@invalid@");

        match result {
            Err(JsonError::UnexpectedToken { .. }) => Ok(()), // Expected
            _ => panic!("Expected UnexpectedToken error"),
        }
    }
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

    #[test]
    fn test_parse_string_with_newline() -> Result<()> {
        let mut parser = JsonParser::new(r#""hello\nworld""#)?;
        let value = parser.parse()?;
        assert_eq!(value, JsonValue::String("hello\nworld".to_string()));
        Ok(())
    }

    #[test]
    fn test_parse_string_with_unicode() -> Result<()> {
        let mut parser = JsonParser::new(r#""\u0048\u0065\u006c\u006c\u006f""#)?;
        let value = parser.parse()?;
        assert_eq!(value, JsonValue::String("Hello".to_string()));
        Ok(())
    }

    #[test]
    fn test_parse_complex_escapes() -> Result<()> {
        let mut parser = JsonParser::new(r#""line1\nline2\t\"quoted\"\u0021""#)?;
        let value = parser.parse()?;
        assert_eq!(
            value,
            JsonValue::String("line1\nline2\t\"quoted\"!".to_string())
        );
        Ok(())
    }
    #[test]
    fn test_parse_string_with_tab() -> Result<()> {
        let mut parser = JsonParser::new(r#""col1\tcol2""#)?;
        let value = parser.parse()?;
        assert_eq!(value, JsonValue::String("col1\tcol2".to_string()));
        Ok(())
    }

    #[test]
    fn test_parse_string_with_quotes() -> Result<()> {
        let mut parser = JsonParser::new(r#""say \"hi\"""#)?;
        let value = parser.parse()?;
        assert_eq!(value, JsonValue::String("say \"hi\"".to_string()));
        Ok(())
    }
    #[test]
    fn test_array_accessor() -> Result<()> {
        let mut parser = JsonParser::new("[1, 2, 3]")?;
        let value = parser.parse()?;
        assert_eq!(value.as_array().map(Vec::len), Some(3));
        Ok(())
    }

    #[test]
    fn test_array_get_index() -> Result<()> {
        let mut parser = JsonParser::new("[10, 20, 30]")?;
        let value = parser.parse()?;
        assert_eq!(value.get_index(1), Some(&JsonValue::Number(20.0)));
        assert_eq!(value.get_index(5), None);
        Ok(())
    }
    #[test]
    fn test_parse_empty_object() -> Result<()> {
        let mut parser = JsonParser::new("{}")?;
        let value = parser.parse()?;
        assert_eq!(value, JsonValue::Object(HashMap::new()));
        Ok(())
    }

    #[test]
    fn test_parse_object_single_key() -> Result<()> {
        let mut parser = JsonParser::new(r#"{"key": "value"}"#)?;
        let value = parser.parse()?;
        let mut expected = HashMap::new();
        expected.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(value, JsonValue::Object(expected));
        Ok(())
    }

    #[test]
    fn test_parse_object_multiple_keys() -> Result<()> {
        let mut parser = JsonParser::new(r#"{"name": "Alice", "age": 30}"#)?;
        let value = parser.parse()?;
        if let JsonValue::Object(obj) = value {
            assert_eq!(
                obj.get("name"),
                Some(&JsonValue::String("Alice".to_string()))
            );
            assert_eq!(obj.get("age"), Some(&JsonValue::Number(30.0)));
        } else {
            panic!("Expected object");
        }
        Ok(())
    }
    #[test]
    fn test_object_accessor() -> Result<()> {
        let mut parser = JsonParser::new(r#"{"name": "test"}"#)?;
        let value = parser.parse()?;
        assert_eq!(value.as_object().map(HashMap::len), Some(1));
        Ok(())
    }

    #[test]
    fn test_object_get() -> Result<()> {
        let mut parser = JsonParser::new(r#"{"name": "Alice", "age": 30}"#)?;
        let value = parser.parse()?;
        assert_eq!(
            value.get("name"),
            Some(&JsonValue::String("Alice".to_string()))
        );
        assert_eq!(value.get("missing"), None);
        Ok(())
    }
    #[test]
    fn test_parse_nested_arrays() -> Result<()> {
        let mut parser = JsonParser::new("[[1, 2], [3, 4]]")?;
        let value = parser.parse()?;
        let expected = JsonValue::Array(vec![
            JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)]),
            JsonValue::Array(vec![JsonValue::Number(3.0), JsonValue::Number(4.0)]),
        ]);
        assert_eq!(value, expected);
        Ok(())
    }

    #[test]
    fn test_parse_deeply_nested() -> Result<()> {
        let mut parser = JsonParser::new("[[[1]]]")?;
        let value = parser.parse()?;
        let expected = JsonValue::Array(vec![JsonValue::Array(vec![JsonValue::Array(vec![
            JsonValue::Number(1.0),
        ])])]);
        assert_eq!(value, expected);
        Ok(())
    }

    #[test]
    fn test_parse_nested_object() -> Result<()> {
        let mut parser = JsonParser::new(r#"{"outer": {"inner": 1}}"#)?;
        let value = parser.parse()?;
        if let JsonValue::Object(outer) = value {
            if let Some(JsonValue::Object(inner)) = outer.get("outer") {
                assert_eq!(inner.get("inner"), Some(&JsonValue::Number(1.0)));
            } else {
                panic!("Expected nested object");
            }
        } else {
            panic!("Expected object");
        }
        Ok(())
    }

    #[test]
    fn test_parse_array_in_object() -> Result<()> {
        let mut parser = JsonParser::new(r#"{"items": [1, 2, 3]}"#)?;
        let value = parser.parse()?;
        if let JsonValue::Object(obj) = value {
            if let Some(JsonValue::Array(arr)) = obj.get("items") {
                assert_eq!(arr.len(), 3);
            } else {
                panic!("Expected array");
            }
        } else {
            panic!("Expected object");
        }
        Ok(())
    }

    #[test]
    fn test_parse_object_in_array() -> Result<()> {
        let mut parser = JsonParser::new(r#"[{"a": 1}, {"b": 2}]"#)?;
        let value = parser.parse()?;
        if let JsonValue::Array(arr) = value {
            assert_eq!(arr.len(), 2);
        } else {
            panic!("Expected array");
        }
        Ok(())
    }

    #[test]
    fn test_error_unclosed_array() -> Result<()> {
        let mut parser = JsonParser::new("[1, 2")?;
        let result = parser.parse();
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_error_unclosed_object() -> Result<()> {
        let mut parser = JsonParser::new(r#"{"key": 1"#)?;
        let result = parser.parse();
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_error_trailing_comma_array() -> Result<()> {
        let mut parser = JsonParser::new("[1, 2,]")?;
        let result = parser.parse();
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_error_trailing_comma_object() -> Result<()> {
        let mut parser = JsonParser::new(r#"{"a": 1,}"#)?;
        let result = parser.parse();
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_error_missing_colon() -> Result<()> {
        let mut parser = JsonParser::new(r#"{"key" 1}"#)?;
        let result = parser.parse();
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_error_invalid_key() -> Result<()> {
        let mut parser = JsonParser::new(r#"{123: "value"}"#)?;
        let result = parser.parse();
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_error_missing_comma_array() -> Result<()> {
        let result = JsonParser::new("[1 2 3]")?.parse();
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_error_missing_comma_object() -> Result<()> {
        let mut parser = JsonParser::new(r#"{"a": 1 "b": 2}"#)?;
        let result = parser.parse();
        assert!(result.is_err());
        Ok(())
    }
}
