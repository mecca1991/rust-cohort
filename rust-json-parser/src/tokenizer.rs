use std::collections::HashMap;

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
        println!("These are the chars {:?}", characters);
        Self {
            input: characters,
            position: 0,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, JsonError> {
        let mut tokens: Vec<Token> = Vec::new();
        while let Some(ch) = self.peek() {
            println!("This is matching now {}", &ch);
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
                    let mut closed = false;
                    self.advance();
                    let mut collected = String::new();
                    while self.peek().is_some() {
                        let nchar = self.input[self.position];
                        if nchar == '\n' {
                            self.advance();
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
                    } else {
                        println!("This is the final collection {}", &collected);
                        tokens.push(Token::String(collected));
                        println!("Tokens added {:?}", &tokens);
                    }
                    
                }
                ch if ch.is_numeric() || (ch == '-') || (ch == '.') => {
                    let mut num_string: String = String::new();
                    while let Some(value) = self.peek() {
                        if value.is_numeric()
                            || value.to_string().starts_with("-")
                            || value.to_string().contains(".")
                        {
                            num_string.push(value);
                            self.advance();
                        } else {
                            break;
                        }
                        if num_string.starts_with(".") {
                            return Err(JsonError::UnexpectedToken {
                                position: 0,
                                expected: "JSON Value".to_string(),
                                found: ".".to_string(),
                            });
                        }
                    }
                    if !num_string.is_empty() {
                        let num = num_string.parse();
                        match num {
                            Ok(num) => tokens.push(Token::Number(num)),
                            Err(error) => {
                                return Err(JsonError::InvalidNumber {
                                    value: format!("{error:?}"),
                                    position: self.position,
                                });
                            }
                        }
                    }
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
                    if &match_str == "null" {
                        tokens.push(Token::Null);
                    } else if &match_str == "true" || &match_str == "false" {
                        let matched = match_str.parse();
                        match matched {
                            Ok(item) => tokens.push(Token::Boolean(item)),
                            Err(_error) => {
                                return Err(JsonError::UnexpectedToken {
                                    expected: "true or false boolean value".to_string(),
                                    found: match_str.to_string(),
                                    position: self.position,
                                });
                            }
                        }
                    } else {
                        return Err(JsonError::UnexpectedToken {
                            expected: "Boolean value or Null value".to_string(),
                            found: match_str.to_string(),
                            position: self.position,
                        });
                    }
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
       let next_token = self.input.get(self.position).copied();
       self.position += 1;
       next_token
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()

    }
    fn is_at_end(&self) -> bool {
        if let Some(_ch) = self.input.last() {
            return true;
        } else {
            return false;
        }
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
        let mut tokenizer = Tokenizer::new("{}");
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::LeftBrace);
        assert_eq!(tokens[1], Token::RightBrace);
        Ok(())
    }
    #[test]
    fn test_simple_string() -> Result<()> {
        let mut tokenizer = Tokenizer::new(r#""hello""#);
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("hello".to_string()));
        Ok(())
    }
    #[test]
    fn test_tokenize_string() -> Result<()> {
        let mut tokenizer = Tokenizer::new(r#""hello world""#);
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("hello world".to_string()));
        Ok(())
    }
    #[test]
    fn test_empty_string() -> Result<()> {
        let mut tokenizer = Tokenizer::new(r#""""#);

        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_containing_json_special_chars() -> Result<()> {
        let mut tokenizer = Tokenizer::new(r#""{key: value}""#);
        let tokens = tokenizer.tokenize()?;

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("{key: value}".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_with_keyword_like_content() -> Result<()> {
        let mut tokenizer = Tokenizer::new(r#""not true or false""#);
        let tokens = tokenizer.tokenize()?;

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("not true or false".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_with_number_like_content() -> Result<()> {
        let mut tokenizer = Tokenizer::new(r#""phone: 555-1234""#);
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("phone: 555-1234".to_string()));
        Ok(())
    }

    #[test]
    fn test_number() -> Result<()> {
        let mut tokenizer = Tokenizer::new("42");

        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(42.0));
        Ok(())
    }

    #[test]
    fn test_negative_number() -> Result<()> {
        let mut tokenizer = Tokenizer::new("-42");
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(-42.0));
        Ok(())
    }

    #[test]
    fn test_decimal_number() -> Result<()> {
        let mut tokenizer = Tokenizer::new("0.5");
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(0.5));
        Ok(())
    }

    #[test]
    fn test_leading_decimal_number() -> Result<()> {
        let mut tokenizer = Tokenizer::new(".5");
        let res = tokenizer.tokenize();
        assert!(res.is_err());

        Ok(())
    }

    #[test]
    fn test_boolean_and_null() -> Result<()> {
        let mut tokenizer = Tokenizer::new("true false null");
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Boolean(true));
        assert_eq!(tokens[1], Token::Boolean(false));
        assert_eq!(tokens[2], Token::Null);
        Ok(())
    }

    #[test]
    fn test_simple_object() -> Result<()> {
        let mut tokenizer = Tokenizer::new(r#"{"name": "Alice"}"#);
        let tokens = tokenizer.tokenize()?;

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
        let mut tokenizer = Tokenizer::new(r#"{"age": 30, "active": true}"#);
        let tokens = tokenizer.tokenize()?;
        assert!(tokens.contains(&Token::String("age".to_string())));
        assert!(tokens.contains(&Token::Number(30.0)));
        assert!(tokens.contains(&Token::Comma));
        assert!(tokens.contains(&Token::String("active".to_string())));
        assert!(tokens.contains(&Token::Boolean(true)));
        Ok(())
    }

    #[test]
    fn test_json_with_brackets() -> Result<()> {
        let mut tokenizer = Tokenizer::new(r#"{"age": 30, "children_names": ["Naia", "Bryan"]}"#);
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 13);
        assert_eq!(tokens[7], Token::LeftBracket);
        assert_eq!(tokens[11], Token::RightBracket);
        Ok(())
    }
    #[test]
    fn test_unterminated_string_does_not_panic() -> Result<()> {
        let mut tokenizer = Tokenizer::new(r#""hello"#);
        let tokens = tokenizer.tokenize();
        assert!(tokens.is_err());
        Ok(())
    }
    #[test]
    fn test_leading_decimal_not_a_number() {
        // .5 is invalid JSON - numbers must have leading digit (0.5 is valid)
        let mut tokenizer = Tokenizer::new(".5");
        let err = tokenizer.tokenize().unwrap_err();
        assert!(matches!(
            err,
            JsonError::UnexpectedToken { position: 0, .. }
        ));
    }
    #[test]
    fn test_keyword_does_not_swallow_following_tokens() -> Result<()> {
        let mut tokenizer = Tokenizer::new("[true, null]");
        let tokens = tokenizer.tokenize()?;
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
            let mut tokenizer = Tokenizer::new(input);

            assert!(
                matches!(tokenizer.tokenize(), Err(JsonError::UnexpectedToken { .. })),
                "Should fail for: {}",
                input
            );
        }
    }

    #[test]
    fn test_malformed_number_is_error_not_panic() {
        for input in ["-", "1-2", "1.2.3", "--5"] {
            let mut tokenizer = Tokenizer::new(input);
            assert!(
                matches!(tokenizer.tokenize(), Err(JsonError::InvalidNumber { .. })),
                "Should fail for: {}",
                input
            );
        }
    }
    #[test]
    fn test_unterminated_string() {
        let mut tokenizer = Tokenizer::new(r#""missing end quote"#);

        let err = tokenizer.tokenize().unwrap_err();
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
            let mut tokenizer = Tokenizer::new(input);
            assert!(tokenizer.tokenize().is_err(), "Should fail for: {}", input);
        }
    }
    #[test]
    fn test_tokenizer_struct_creation() {
        let _tokenizer = Tokenizer::new(r#""hello""#);
        // Tokenizer should be created without error
        // Internal state is private, so we test via tokenize()
    }
    #[test]
    fn test_tokenize_number() -> Result<()> {
        let mut tokenizer = Tokenizer::new("42");
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens, vec![Token::Number(42.0)]);
        Ok(())
    }

    #[test]
    fn test_tokenize_literals() -> Result<()> {
        let mut t1 = Tokenizer::new("true");
        assert_eq!(t1.tokenize()?, vec![Token::Boolean(true)]);

        let mut t2 = Tokenizer::new("false");
        assert_eq!(t2.tokenize()?, vec![Token::Boolean(false)]);

        let mut t3 = Tokenizer::new("null");
        assert_eq!(t3.tokenize()?, vec![Token::Null]);
        Ok(())
    }

    #[test]
    fn test_tokenize_simple_string() -> Result<()> {
        let mut tokenizer = Tokenizer::new(r#""hello""#);
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens, vec![Token::String("hello".to_string())]);
        Ok(())
    }
    #[test]
    fn test_tokenizer_multiple_tokens() -> Result<()> {
        // Tests that a single tokenize() call handles multiple tokens
        // Note: Unlike Python iterators, calling tokenize() again on the same
        // instance would return empty - the input has been consumed.
        // Create a new Tokenizer instance if you need to parse new input.
        let mut tokenizer = Tokenizer::new("123 456");
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens.len(), 2);
        Ok(())
    }

    #[test]
    fn test_tokenize_negative_number() -> Result<()> {
        let mut tokenizer = Tokenizer::new("-3.14");
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens, vec![Token::Number(-3.14)]);
        Ok(())
    }

    #[test]
    fn test_invalid_keyword_error_position_points_to_start() -> Result<()> {
        let input = "   xyz";
        let mut tokenizer = Tokenizer::new(input);
        let err = tokenizer.tokenize().unwrap_err();
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
        let mut tokenizer = Tokenizer::new(r#""hello\nworld""#);
        let tokens = tokenizer.tokenize()?;
        assert_eq!(tokens, vec![Token::String("hello\nworld".to_string())]);
        Ok(())
    }

    // #[test]
    // fn test_escape_tab() -> Result<()> {
    //     let mut tokenizer = Tokenizer::new(r#""col1\tcol2""#);
    //     let tokens = tokenizer.tokenize()?;
    //     assert_eq!(tokens, vec![Token::String("col1\tcol2".to_string())]);
    //     Ok(())
    // }

    // #[test]
    // fn test_escape_quote() -> Result<()> {
    //     let mut tokenizer = Tokenizer::new(r#""say \"hello\"""#);
    //     let tokens = tokenizer.tokenize()?;
    //     assert_eq!(tokens, vec![Token::String("say \"hello\"".to_string())]);
    //     Ok(())
    // }

    // #[test]
    // fn test_escape_backslash() -> Result<()> {
    //     let mut tokenizer = Tokenizer::new(r#""path\\to\\file""#);
    //     let tokens = tokenizer.tokenize()?;
    //     assert_eq!(tokens, vec![Token::String("path\\to\\file".to_string())]);
    //     Ok(())
    // }

    // #[test]
    // fn test_multiple_escapes() -> Result<()> {
    //     let mut tokenizer = Tokenizer::new(r#""a\nb\tc\"""#);
    //     let tokens = tokenizer.tokenize()?;
    //     assert_eq!(tokens, vec![Token::String("a\nb\tc\"".to_string())]);
    //     Ok(())
    // }
}
