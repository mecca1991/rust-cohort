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
        let parsed_data = self.parse_value()?;

        if !self.is_at_end() {
            let token = self.tokens.get(self.position);
            return Err(JsonError::UnexpectedToken {
                expected: "End of input".to_string(),
                found: format!("{:?}", token),
                position: self.position,
            });
        }
        Ok(parsed_data)
    }

    pub fn parse_value(&mut self) -> Result<JsonValue> {
        let token = self.tokens.get(self.position).cloned();
        if let Some(token) = token {
            match token {
                Token::Boolean(value) => {
                    self.advance();
                    Ok(JsonValue::Boolean(value))
                }
                Token::String(value) => {
                    self.advance();
                    Ok(JsonValue::String(value))
                }
                Token::Number(value) => {
                    self.advance();
                    Ok(JsonValue::Number(value))
                }
                Token::LeftBracket => {
                    self.advance();
                    self.parse_array()
                }
                Token::LeftBrace => {
                    self.advance();
                    self.parse_object()
                }
                Token::Null => {
                    self.advance();
                    Ok(JsonValue::Null)
                }
                _ => Err(JsonError::UnexpectedToken {
                    expected: "JSON value".to_string(),
                    found: format!("{:?}", token),
                    position: self.position,
                }),
            }
        } else {
            Err(JsonError::UnexpectedEndOfInput {
                expected: "JSON value".to_string(),
                position: self.position,
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

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position + 1)
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }
    fn get_jsonvalue_string(&self, jsonvalue: JsonValue) -> Result<String> {
        let result = match jsonvalue {
            JsonValue::String(key) => key,
            unexpected => {
                return Err(JsonError::UnexpectedToken {
                    expected: "JSON value".to_string(),
                    found: format!("{:?}", unexpected),
                    position: self.position,
                });
            }
        };
        Ok(result)
    }
    fn is_valid_before_comma_array(&self, token: Option<&Token>) -> bool {
        matches!(
            token,
            Some(
                Token::Number(_)
                    | Token::Boolean(_)
                    | Token::String(_)
                    | Token::RightBracket
                    | Token::RightBrace
                    | Token::Null
            )
        )
    }
    fn is_valid_after_comma_array(&self, token: Option<&Token>) -> bool {
        matches!(
            token,
            Some(
                Token::Number(_)
                    | Token::Boolean(_)
                    | Token::String(_)
                    | Token::LeftBracket
                    | Token::LeftBrace
                    | Token::Null
            )
        )
    }

    fn parse_object(&mut self) -> Result<JsonValue> {
        let mut hash_items: HashMap<String, JsonValue> = HashMap::new();
        let mut current_key = String::new();
        while let Some(token) = self.tokens.get(self.position).cloned() {
            match token {
                Token::RightBrace => {
                    self.advance();
                    return Ok(JsonValue::Object(hash_items));
                }
                Token::Colon => {
                    // let key = &current_key.chars().take(0).next();
                    // match key {
                    //     None => {
                    //         return Err(JsonError::UnexpectedToken {
                    //             expected: "String".to_string(),
                    //             found: ":".to_string(),
                    //             position: self.position,
                    //         });
                    //     },
                    //     _ => {
                            
                    //     }
                    // }
                    self.advance();
                    let value = self.parse_value()?;
                    hash_items.insert(current_key.clone(), value);
                    // position moved to next after self.advance
                    let next_token = self.tokens.get(self.position);

                    match next_token {
                        Some(Token::Comma) | Some(Token::RightBrace) => {
                            continue;
                        }
                        Some(token) => {
                            return Err(JsonError::UnexpectedToken {
                                expected: "',' or '}'".to_string(),
                                found: format!("{token:?}"),
                                position: self.position,
                            });
                        }
                        None => {
                            return Err(JsonError::UnexpectedEndOfInput {
                                expected: "',' or '}'".to_string(),
                                position: self.position,
                            });
                        }
                    }
                }
                Token::Comma => {
                    if hash_items.is_empty() {
                        return Err(JsonError::UnexpectedToken {
                            expected: "String".to_string(),
                            found: ",".to_string(),
                            position: self.position,
                        });
                    }
                    match self.peek() {
                        Some(token) => {
                            if !matches!(Some(token), Some(Token::String(_))) {
                                return Err(JsonError::UnexpectedToken {
                                    expected: "String".to_string(),
                                    found: format!("{token:?}"),
                                    position: self.position + 1,
                                });
                            } else {
                                self.advance();
                            }
                        }
                        None => {
                            return Err(JsonError::UnexpectedEndOfInput {
                                expected: "String".to_string(),
                                position: self.position + 1,
                            });
                        }
                    }
                    let parsed_value = self.parse_value()?;
                    current_key = self.get_jsonvalue_string(parsed_value)?;

                    match self.tokens.get(self.position) {
                        Some(Token::Colon) => {}
                        Some(token) => {
                            return Err(JsonError::UnexpectedToken {
                                expected: "':'".to_string(),
                                found: format!("{token:?}"),
                                position: self.position,
                            });
                        }
                        None => {
                            return Err(JsonError::UnexpectedEndOfInput {
                                expected: "':'".to_string(),
                                position: self.position,
                            });
                        }
                    }
                }
                _ => {
                    let parsed_key = self.parse_value()?;
                    current_key = self.get_jsonvalue_string(parsed_key)?;

                    match self.tokens.get(self.position) {
                        Some(Token::Colon) => {
                            continue;
                        }
                        Some(token) => {
                            return Err(JsonError::UnexpectedToken {
                                expected: "':'".to_string(),
                                found: format!("{token:?}"),
                                position: self.position,
                            });
                        }
                        None => {
                            return Err(JsonError::UnexpectedEndOfInput {
                                expected: "':'".to_string(),
                                position: self.position,
                            });
                        }
                    }
                }
            }
        }
        Err(JsonError::UnexpectedEndOfInput {
            expected: "'}' ".to_string(),
            position: self.position,
        })
    }

    fn parse_array(&mut self) -> Result<JsonValue> {
        let mut array_data: Vec<JsonValue> = Vec::new();
        while let Some(token) = self.tokens.get(self.position).cloned() {
            match token {
                Token::RightBracket => {
                    self.advance();
                    return Ok(JsonValue::Array(array_data));
                }
                Token::Comma => {
                    let current_position = &self.position;
                    let previous_position = current_position.checked_sub(1);
                    match previous_position {
                        Some(position) => {
                            let previous_token = self.tokens.get(position);
                            let next_token = self.peek();

                            let previous_valid = self.is_valid_before_comma_array(previous_token);
                            let next_valid = self.is_valid_after_comma_array(next_token);

                            if !previous_valid || !next_valid {
                                return Err(JsonError::UnexpectedToken {
                                    expected: "], }, or Json value".to_string(),
                                    found: "Invalid".to_string(),
                                    position: self.position,
                                });
                            } else {
                                self.advance();
                            }
                        }
                        None => {
                            return Err(JsonError::UnexpectedToken {
                                expected: "], }, or Json value".to_string(),
                                found: format!("{:?}", token),
                                position: self.position,
                            });
                        }
                    }
                }
                _ => {
                    array_data.push(self.parse_value()?);
                    let current_token = self.tokens.get(self.position);
                    match current_token {
                        Some(Token::Comma) => {
                            continue;
                        }
                        Some(Token::RightBracket) => {
                            continue;
                        }
                        None => {
                            return Err(JsonError::UnexpectedEndOfInput {
                                expected: "JSON value".to_string(),
                                position: self.position,
                            });
                        }
                        _ => {
                            return Err(JsonError::UnexpectedToken {
                                expected: ",".to_string(),
                                found: format!("{:?}", current_token),
                                position: self.position,
                            });
                        }
                    }
                }
            }
        }
        Err(JsonError::UnexpectedEndOfInput {
            expected: "Valid Json".to_string(),
            position: self.position,
        })
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
}

#[cfg(test)]
mod array_tests {
    use super::*;

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
    fn test_error_missing_comma_array() -> Result<()> {
        let result = JsonParser::new("[1 2 3]")?.parse();
        assert!(result.is_err());
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
    fn test_array_get_index() -> Result<()> {
        let mut parser = JsonParser::new("[10, 20, 30]")?;
        let value = parser.parse()?;
        assert_eq!(value.get_index(1), Some(&JsonValue::Number(20.0)));
        assert_eq!(value.get_index(5), None);
        Ok(())
    }

    #[test]
    fn test_error_trailing_comma_array() -> Result<()> {
        let mut parser = JsonParser::new("[1, 2,]")?;
        let result = parser.parse();
        assert!(result.is_err());
        Ok(())
    }
}

#[cfg(test)]
mod object_tests {
    use super::*;

    #[test]
    fn test_error_unclosed_object() -> Result<()> {
        let mut parser = JsonParser::new(r#"{"key": 1"#)?;
        let result = parser.parse();
        assert!(result.is_err());
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
    fn test_error_missing_comma_object() -> Result<()> {
        let mut parser = JsonParser::new(r#"{"a": 1 "b": 2}"#)?;
        let result = parser.parse();
        assert!(result.is_err());
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
    fn test_error_leading_colon_object() -> Result<()> {
        let result = JsonParser::new("{: 1}")?.parse();
        assert!(matches!(result, Err(JsonError::UnexpectedToken { .. })));
        Ok(())
    }
}

#[cfg(test)]
mod error_tests {
    use super::*;

    /// Bug: `{"a": 1, "b"}` parses as `Ok({"a": 1})`. Key `"b"` is silently dropped.
    /// Where: `parse_object`, the `Comma` arm reads the next key but never requires `:` after it,
    /// and the `_ => self.advance()` catch-all skips any token it doesn't expect.
    /// Fix: read the object top to bottom like the grammar: consume `{`; if `}`, return empty;
    /// otherwise loop: require a string key, require `:`, `parse_value()`, insert, then require
    /// `,` (continue) or `}` (break). Any other token at any step is an `UnexpectedToken` error.
    #[test]
    fn test_error_key_without_colon_after_comma() -> Result<()> {
        let result = JsonParser::new(r#"{"a": 1, "b"}"#)?.parse();
        assert!(matches!(result, Err(JsonError::UnexpectedToken { .. })));
        Ok(())
    }

    /// Bug: `{"a"` parses as `Ok({"a": null})`, a value that isn't in the input.
    /// Where: `parse_object`, the `LeftBrace` arm inserts the key with a `JsonValue::Null`
    /// placeholder before `:` and the value are seen, and `while let Some(..)` exits normally
    /// when tokens run out.
    /// Fix: with the grammar-shaped loop, insert only after key, `:` and value have all parsed,
    /// so `current_key` and the placeholder go away. Running out of tokens at any step is an
    /// `UnexpectedEndOfInput` error.
    #[test]
    fn test_error_key_without_value() -> Result<()> {
        let result = JsonParser::new(r#"{"a""#)?.parse();
        assert!(matches!(
            result,
            Err(JsonError::UnexpectedEndOfInput { .. })
        ));
        Ok(())
    }

    /// Bug: `{` parses as `Ok({})`.
    /// Where: `parse_object`, the `while let Some(token)` loop ends when tokens run out and
    /// falls through to `Ok(JsonValue::Object(..))`.
    /// Fix: an object only ends at `}`. Reaching the end of tokens inside an object is an
    /// `UnexpectedEndOfInput` error. Consuming `{` in `parse_value` (as you already do for `[`)
    /// keeps both parsers on one convention.
    #[test]
    fn test_error_lone_open_brace() -> Result<()> {
        let result = JsonParser::new("{")?.parse();
        assert!(matches!(
            result,
            Err(JsonError::UnexpectedEndOfInput { .. })
        ));
        Ok(())
    }

    /// Bug: `[1,,2]` parses as `Ok([1, 2])`.
    /// Where: `parse_array`, the `Comma` arm only checks that `]` doesn't follow the comma,
    /// not that a value came before it or follows it.
    /// Fix: check for `]` once at the start (empty array), then loop:
    /// `push(parse_value()?)`, then `,` (continue) or `]` (break), anything else is an error.
    /// A comma then always has a value on both sides, and `parse_value` rejects the second `,`.
    #[test]
    fn test_error_double_comma_array() -> Result<()> {
        let result = JsonParser::new("[1,,2]")?.parse();
        assert!(matches!(result, Err(JsonError::UnexpectedToken { .. })));
        Ok(())
    }

    /// Bug: `[,1]` parses as `Ok([1])`.
    /// Where: `parse_array`, same `Comma` arm as `test_error_double_comma_array`.
    /// Fix: same "one value, then one separator" loop. The first element goes through
    /// `parse_value`, which returns `UnexpectedToken` for a `,` (fix its swapped variants too).
    #[test]
    fn test_error_leading_comma_array() -> Result<()> {
        let result = JsonParser::new("[,1]")?.parse();
        assert!(matches!(result, Err(JsonError::UnexpectedToken { .. })));
        Ok(())
    }

    /// Bug: `[[1] 2]` parses as `Ok([[1], 2])`.
    /// Where: `parse_array`, the `LeftBracket` arm parses a nested array itself and skips the
    /// separator check that other elements get.
    /// Fix: remove that arm. `parse_value` already handles `[`, so nested arrays go through
    /// the same "value, then `,` or `]`" loop.
    #[test]
    fn test_error_missing_comma_after_nested_array() -> Result<()> {
        let result = JsonParser::new("[[1] 2]")?.parse();
        assert!(matches!(result, Err(JsonError::UnexpectedToken { .. })));
        Ok(())
    }

    /// Bug: `[1] 2` parses as `Ok([1])`. A regression: your week 3 `parse` rejected extra tokens.
    /// Where: `parse` only calls `parse_value` and ignores whatever is left.
    /// Fix: after `parse_value`, return `UnexpectedToken` unless `is_at_end()`. Keep this in
    /// `parse`, not `parse_value`, or every array element after the first gets rejected.
    #[test]
    fn test_error_trailing_tokens() -> Result<()> {
        let result = JsonParser::new("[1] 2")?.parse();
        assert!(matches!(result, Err(JsonError::UnexpectedToken { .. })));
        Ok(())
    }
    #[test]
    fn test_parse_empty_array() -> Result<()> {
        let mut parser = JsonParser::new("[]")?;
        let value = parser.parse()?;
        assert_eq!(value, JsonValue::Array(vec![]));
        Ok(())
    }

    #[test]
    fn test_parse_array_single() -> Result<()> {
        let mut parser = JsonParser::new("[1]")?;
        let value = parser.parse()?;
        assert_eq!(value, JsonValue::Array(vec![JsonValue::Number(1.0)]));
        Ok(())
    }

    #[test]
    fn test_parse_array_multiple() -> Result<()> {
        let mut parser = JsonParser::new("[1, 2, 3]")?;
        let value = parser.parse()?;
        let expected = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0),
        ]);
        assert_eq!(value, expected);
        Ok(())
    }

    #[test]
    fn test_parse_array_mixed_types() -> Result<()> {
        let mut parser = JsonParser::new(r#"[1, "two", true, null]"#)?;
        let value = parser.parse()?;
        let expected = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::String("two".to_string()),
            JsonValue::Boolean(true),
            JsonValue::Null,
        ]);
        assert_eq!(value, expected);
        Ok(())
    }
}
