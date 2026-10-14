use std::char;

use crate::error::JsonError;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tokenizer {
    input: Vec<char>,
    position: usize,
}

impl Tokenizer {
    pub fn new(input: &str) -> Self {
        let characters = input.chars().collect();
        Self {
            input: characters,
            position: 0,
        }
    }

    fn get_real_char(&mut self, ch: char) -> Result<char, JsonError> {
        match ch {
            'n' => {
                self.advance();
                Ok('\n')
            }
            't' => {
                self.advance();
                Ok('\t')
            }
            'b' => {
                self.advance();
                Ok('\x08')
            }
            'f' => {
                self.advance();
                Ok('\x0C')
            }
            'r' => {
                self.advance();
                Ok('\r')
            }
            '"' => {
                self.advance();
                Ok('\"')
            }
            '\\' => {
                self.advance();
                Ok(ch)
            }
            '/' => {
                self.advance();
                Ok(ch)
            }
            'u' => {
                let mut hex_value = String::new();
                self.advance();
                while let Some(ch) = self.peek() {
                    if hex_value.len() >= 4 {
                        break;
                    }
                    if !ch.is_ascii_hexdigit() {
                        return Err(JsonError::InvalidUnicode {
                            sequence: ch.to_string(),
                            position: self.position,
                        });
                    }
                    hex_value.push(ch);
                    self.advance();
                }
                if hex_value.len() != 4 {
                    return Err(JsonError::InvalidUnicode {
                        sequence: hex_value.to_string(),
                        position: self.position,
                    });
                }
                let st: Result<u32, std::num::ParseIntError> = u32::from_str_radix(&hex_value, 16);
                match st {
                    Ok(st) => {
                        if let Some(ch) = char::from_u32(st) {
                            Ok(ch)
                        } else {
                            Err(JsonError::InvalidUnicode {
                                sequence: hex_value.to_string(),
                                position: self.position,
                            })
                        }
                    }
                    Err(_) => Err(JsonError::InvalidUnicode {
                        sequence: hex_value.to_string(),
                        position: self.position,
                    }),
                }
            }
            _ => Err(JsonError::InvalidEscape {
                char: ch,
                position: self.position,
            }),
        }
    }
    fn tokenize_boolean_string(&mut self, matched_str: &str) -> Result<Token, JsonError> {
        match matched_str {
            "null" => Ok(Token::Null),
            "true" => Ok(Token::Boolean(true)),
            "false" => Ok(Token::Boolean(false)),
            _ => Err(JsonError::UnexpectedToken {
                expected: "Boolean value or Null value".to_string(),
                found: matched_str.to_string(),
                position: self.position,
            }),
        }
    }
    fn tokenize_numeric_string(&mut self) -> Result<Token, JsonError> {
        let mut num_string: String = String::new();
        while let Some(value) = self.peek() {
            if !(value.is_numeric()
                || value.to_string().starts_with("-")
                || value.to_string().contains("."))
            {
                break;
            }
            num_string.push(value);
            self.advance();
        }
        if num_string.is_empty() || num_string.starts_with(".") {
            return Err(JsonError::UnexpectedToken {
                position: 0,
                expected: "JSON Value".to_string(),
                found: ".".to_string(),
            });
        }
        let res = num_string.parse::<f64>();
        match res {
            Ok(item) => Ok(Token::Number(item)),
            Err(err) => Err(JsonError::InvalidNumber {
                value: format!("{err:?}"),
                position: self.position,
            }),
        }
    }
    pub fn tokenize(&mut self) -> Result<Vec<Token>, JsonError> {
        let mut tokens: Vec<Token> = Vec::new();
        while let Some(ch) = self.peek() {
            match ch {
                '{' => {
                    tokens.push(Token::LeftBrace);
                    self.advance();
                }
                '}' => {
                    tokens.push(Token::RightBrace);
                    self.advance();
                }
                '[' => {
                    tokens.push(Token::LeftBracket);
                    self.advance();
                }
                ']' => {
                    tokens.push(Token::RightBracket);
                    self.advance();
                }
                ':' => {
                    tokens.push(Token::Colon);
                    self.advance();
                }
                ',' => {
                    tokens.push(Token::Comma);
                    self.advance();
                }
                '"' => {
                    self.advance();
                    let mut closed = false;
                    let mut collected = String::new();
                    while let Some(nchar) = self.peek() {
                        if nchar == '\\' {
                            self.advance();
                            if let Some(ch) = self.peek() {
                                let cha = self.get_real_char(ch)?;
                                collected.push(cha);
                            }
                            continue;
                        }

                        self.advance();
                        if nchar == '"' {
                            closed = true;
                            break;
                        }
                        collected.push(nchar);
                    }
                    if !closed {
                        return Err(JsonError::UnexpectedEndOfInput {
                            expected: "JSON value".to_string(),
                            position: 0,
                        });
                    }
                    tokens.push(Token::String(collected));
                }
                ch if ch.is_numeric() || (ch == '-') || (ch == '.') => {
                    let token = self.tokenize_numeric_string()?;
                    tokens.push(token);
                }
                ch if ch == 't' || ch == 'f' || ch == 'n' => {
                    let mut match_str = String::new();
                    while let Some(nchar) = self.peek() {
                        if !nchar.is_alphanumeric() {
                            break;
                        }
                        match_str.push(nchar);
                        self.advance();
                    }
                    let token = self.tokenize_boolean_string(&match_str)?;
                    tokens.push(token);
                }
                ' ' | '\n' | '\r' | '\t' => {
                    self.advance();
                }
                _ => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "valid JSON token".to_string(),
                        found: ch.to_string(),
                        position: self.position,
                    });
                }
            }
        }
        Ok(tokens)
    }
    
    fn advance(&mut self) -> Option<char> {
        let token = self.input.get(self.position).copied();
        if self.is_at_end() {
            token
        } else {
            self.position += 1;
            token
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }
    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Result type alias for cleaner test signatures
    type Result<T> = std::result::Result<T, JsonError>;

    // Tests will be added at each step below.
    // String boundary tests - verify inner vs outer quote handling

    #[test]
    fn test_empty_braes() -> Result<()> {
        let tokens = Tokenizer::new("{}").tokenize()?;
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::LeftBrace);
        assert_eq!(tokens[1], Token::RightBrace);
        Ok(())
    }
    #[test]
    fn test_simple_string() -> Result<()> {
        let tokens = Tokenizer::new(r#""hello""#).tokenize()?;

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("hello".to_string()));
        Ok(())
    }
    #[test]
    fn test_tokenize_string() -> Result<()> {
        let tokens = Tokenizer::new(r#""hello world""#).tokenize()?;

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("hello world".to_string()));
        Ok(())
    }
    #[test]
    fn test_empty_string() -> Result<()> {
        let tokens = Tokenizer::new(r#""""#).tokenize()?;

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_containing_json_special_chars() -> Result<()> {
        let tokens = Tokenizer::new(r#""{key: value}""#).tokenize()?;

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("{key: value}".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_with_keyword_like_content() -> Result<()> {
        let tokens = Tokenizer::new(r#""not true or false""#).tokenize()?;

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("not true or false".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_with_number_like_content() -> Result<()> {
        let tokens = Tokenizer::new(r#""phone: 555-1234""#).tokenize()?;

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("phone: 555-1234".to_string()));
        Ok(())
    }

    #[test]
    fn test_number() -> Result<()> {
        let tokens = Tokenizer::new("42").tokenize()?;

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(42.0));
        Ok(())
    }

    #[test]
    fn test_negative_number() -> Result<()> {
        let tokens = Tokenizer::new("-42").tokenize()?;

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(-42.0));
        Ok(())
    }

    #[test]
    fn test_decimal_number() -> Result<()> {
        let tokens = Tokenizer::new("0.5").tokenize()?;

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(0.5));
        Ok(())
    }

    #[test]
    fn test_leading_decimal_number() -> Result<()> {
        let res = Tokenizer::new(".5").tokenize();
        assert!(res.is_err());

        Ok(())
    }

    #[test]
    fn test_boolean_and_null() -> Result<()> {
        let tokens = Tokenizer::new("true false null").tokenize()?;

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Boolean(true));
        assert_eq!(tokens[1], Token::Boolean(false));
        assert_eq!(tokens[2], Token::Null);
        Ok(())
    }

    #[test]
    fn test_simple_object() -> Result<()> {
        let tokens = Tokenizer::new(r#"{"name": "Alice"}"#).tokenize()?;

        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::LeftBrace);
        assert_eq!(tokens[1], Token::String("name".to_string()));
        assert_eq!(tokens[2], Token::Colon);
        assert_eq!(tokens[3], Token::String("Alice".to_string()));
        assert_eq!(tokens[4], Token::RightBrace);
        Ok(())
    }

    #[test]
    fn test_multiple_values() -> Result<()> {
        let tokens = Tokenizer::new(r#"{"age": 30, "active": true}"#).tokenize()?;

        assert!(tokens.contains(&Token::String("age".to_string())));
        assert!(tokens.contains(&Token::Number(30.0)));
        assert!(tokens.contains(&Token::Comma));
        assert!(tokens.contains(&Token::String("active".to_string())));
        assert!(tokens.contains(&Token::Boolean(true)));
        Ok(())
    }

    #[test]
    fn test_json_with_brackets() -> Result<()> {
        let tokens =
            Tokenizer::new(r#"{"age": 30, "children_names": ["Naia", "Bryan"]}"#).tokenize()?;

        assert_eq!(tokens.len(), 13);
        assert_eq!(tokens[7], Token::LeftBracket);
        assert_eq!(tokens[11], Token::RightBracket);
        Ok(())
    }
    #[test]
    fn test_unterminated_string_does_not_panic() -> Result<()> {
        let tokens = Tokenizer::new(r#""hello"#).tokenize();
        assert!(tokens.is_err());
        Ok(())
    }
    #[test]
    fn test_leading_decimal_not_a_number() {
        // .5 is invalid JSON - numbers must have leading digit (0.5 is valid)
        let err = Tokenizer::new(".5").tokenize().unwrap_err();
        assert!(matches!(
            err,
            JsonError::UnexpectedToken { position: 0, .. }
        ));
    }
    #[test]
    fn test_keyword_does_not_swallow_following_tokens() -> Result<()> {
        let tokens = Tokenizer::new("[true, null]").tokenize()?;

        assert_eq!(
            tokens,
            vec![
                Token::LeftBracket,
                Token::Boolean(true),
                Token::Comma,
                Token::Null,
                Token::RightBracket,
            ]
        );
        Ok(())
    }

    #[test]
    fn test_invalid_keyword_is_error() {
        for input in ["nul", "tru", "fals", "txyz", "true123", "false.452"] {
            let res = Tokenizer::new(input).tokenize();

            assert!(
                matches!(res, Err(JsonError::UnexpectedToken { .. })),
                "Should fail for: {}",
                input
            );
        }
    }

    #[test]
    fn test_malformed_number_is_error_not_panic() {
        for input in ["-", "1-2", "1.2.3", "--5"] {
            let res = Tokenizer::new(input).tokenize();
            assert!(
                matches!(res, Err(JsonError::InvalidNumber { .. })),
                "Should fail for: {}",
                input
            );
        }
    }
    #[test]
    fn test_unterminated_string() {
        let err = Tokenizer::new(r#""missing end quote"#)
            .tokenize()
            .unwrap_err();

        match err {
            JsonError::UnexpectedEndOfInput { position, .. } => {
                assert_eq!(position, 0);
            }
            other => panic!("expected UnexpectedEndOfInput, got {:?}", other),
        }
    }
    #[test]
    fn test_non_ascii_digit_is_error_not_panic() {
        for input in ["½", "٣"] {
            let res = Tokenizer::new(input).tokenize();
            assert!(res.is_err(), "Should fail for: {}", input);
        }
    }
    #[test]
    fn test_tokens_struct_creation() {
        let _tokens = Tokenizer::new(r#""hello""#);
        // Tokenizer should be created without error
        // Internal state is private, so we test via tokenize()
    }
    #[test]
    fn test_tokenize_number() -> Result<()> {
        let tokens = Tokenizer::new("42").tokenize()?;

        assert_eq!(tokens, vec![Token::Number(42.0)]);
        Ok(())
    }

    #[test]
    fn test_tokenize_literals() -> Result<()> {
        let t1 = Tokenizer::new("true").tokenize()?;
        assert_eq!(t1, vec![Token::Boolean(true)]);

        let t2 = Tokenizer::new("false").tokenize()?;
        assert_eq!(t2, vec![Token::Boolean(false)]);

        let t3 = Tokenizer::new("null").tokenize()?;
        assert_eq!(t3, vec![Token::Null]);
        Ok(())
    }

    #[test]
    fn test_tokenize_simple_string() -> Result<()> {
        let tokens = Tokenizer::new(r#""hello""#).tokenize()?;
        assert_eq!(tokens, vec![Token::String("hello".to_string())]);
        Ok(())
    }
    #[test]
    fn test_tokenizer_multiple_tokens() -> Result<()> {
        // Tests that a single tokenize() call handles multiple tokens
        // Note: Unlike Python iterators, calling tokenize() again on the same
        // instance would return empty - the input has been consumed.
        // Create a new Tokenizer instance if you need to parse new input.
        let tokens = Tokenizer::new("123 456").tokenize()?;
        assert_eq!(tokens.len(), 2);
        Ok(())
    }

    #[test]
    fn test_tokenize_negative_number() -> Result<()> {
        let tokens = Tokenizer::new("-3.14").tokenize()?;
        assert_eq!(tokens, vec![Token::Number(-3.14)]);
        Ok(())
    }

    #[test]
    fn test_invalid_keyword_error_position_points_to_start() -> Result<()> {
        let input = "   xyz";
        let err = Tokenizer::new(input).tokenize().unwrap_err();
        match err {
            JsonError::UnexpectedToken { position, .. } => {
                assert_eq!(
                    position, 3,
                    "error position should point to the start of 'xyz' (index 3), not past it"
                );
            }
            other => panic!("expected UnexpectedToken, got {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn test_escape_newline() -> Result<()> {
        let tokens = Tokenizer::new(r#""hello\nworld""#).tokenize()?;

        assert_eq!(tokens, vec![Token::String("hello\nworld".to_string())]);
        Ok(())
    }

    #[test]
    fn test_escape_tab() -> Result<()> {
        let tokens = Tokenizer::new(r#""col1\tcol2""#).tokenize()?;
        assert_eq!(tokens, vec![Token::String("col1\tcol2".to_string())]);
        Ok(())
    }

    #[test]
    fn test_escape_quote() -> Result<()> {
        let tokens = Tokenizer::new(r#""say \"hello\"""#).tokenize()?;
        assert_eq!(tokens, vec![Token::String("say \"hello\"".to_string())]);
        Ok(())
    }

    #[test]
    fn test_escape_backslash() -> Result<()> {
        let tokens = Tokenizer::new(r#""path\\to\\file""#).tokenize()?;
        assert_eq!(tokens, vec![Token::String("path\\to\\file".to_string())]);
        Ok(())
    }

    #[test]
    fn test_multiple_escapes() -> Result<()> {
        let tokens = Tokenizer::new(r#""a\nb\tc\"""#).tokenize()?;
        assert_eq!(tokens, vec![Token::String("a\nb\tc\"".to_string())]);
        Ok(())
    }
    #[test]
    fn test_escape_forward_slash() -> Result<()> {
        let tokens = Tokenizer::new(r#""a\/b""#).tokenize()?;
        assert_eq!(tokens, vec![Token::String("a/b".to_string())]);
        Ok(())
    }

    #[test]
    fn test_escape_carriage_return() -> Result<()> {
        let tokens = Tokenizer::new(r#""line\r\n""#).tokenize()?;
        assert_eq!(tokens, vec![Token::String("line\r\n".to_string())]);
        Ok(())
    }

    #[test]
    fn test_escape_backspace_formfeed() -> Result<()> {
        let tokens = Tokenizer::new(r#""\b\f""#).tokenize()?;
        assert_eq!(tokens, vec![Token::String("\u{0008}\u{000C}".to_string())]);
        Ok(())
    }
    #[test]
    fn test_unicode_escape_basic() -> Result<()> {
        // \u0041 is 'A'
        let tokens = Tokenizer::new(r#""\u0041""#).tokenize()?;
        assert_eq!(tokens, vec![Token::String("A".to_string())]);
        Ok(())
    }

    #[test]
    fn test_unicode_escape_multiple() -> Result<()> {
        // \u0048\u0069 is "Hi"
        let tokens = Tokenizer::new(r#""\u0048\u0069""#).tokenize()?;
        assert_eq!(tokens, vec![Token::String("Hi".to_string())]);
        Ok(())
    }

    #[test]
    fn test_unicode_escape_mixed() -> Result<()> {
        // Mix of regular chars and unicode escapes
        let tokens = Tokenizer::new(r#""Hello \u0057orld""#).tokenize()?;
        assert_eq!(tokens, vec![Token::String("Hello World".to_string())]);
        Ok(())
    }

    #[test]
    fn test_unicode_escape_lowercase() -> Result<()> {
        // Lowercase hex digits should work too
        let tokens = Tokenizer::new(r#""\u004a""#).tokenize()?;
        assert_eq!(tokens, vec![Token::String("J".to_string())]);
        Ok(())
    }
    #[test]
    fn test_invalid_unicode_too_short() {
        let mut tokenizer = Tokenizer::new(r#""\u004""#);
        let result = tokenizer.tokenize();
        assert!(matches!(result, Err(JsonError::InvalidUnicode { .. })));
    }

    #[test]
    fn test_invalid_unicode_runs_past_end_of_input() {
        let mut tokenizer = Tokenizer::new(r#""\u00""#);
        let result = tokenizer.tokenize();
        assert!(matches!(result, Err(JsonError::InvalidUnicode { .. })));
    }
    #[test]
    fn test_invalid_escape_sequence() {
        let mut tokenizer = Tokenizer::new(r#""\q""#);
        let result = tokenizer.tokenize();
        assert!(matches!(result, Err(JsonError::InvalidEscape { .. })));
    }

    #[test]
    fn test_invalid_unicode_bad_hex() {
        let mut tokenizer = Tokenizer::new(r#""\u00GG""#);
        let result = tokenizer.tokenize();
        assert!(matches!(result, Err(JsonError::InvalidUnicode { .. })));
    }

    #[test]
    fn test_unterminated_string_with_escape() {
        let mut tokenizer = Tokenizer::new(r#""hello\n"#);
        let result = tokenizer.tokenize();
        assert!(result.is_err());
    }
}
