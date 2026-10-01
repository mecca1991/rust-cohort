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

pub fn tokenize(input: &str) -> Result<Vec<Token>, JsonError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(&ch) = chars.peek() {
        match ch {
            '{' => {
                tokens.push(Token::LeftBrace);
                chars.next();
            }
            '}' => {
                tokens.push(Token::RightBrace);
                chars.next();
            }
            '[' => {
                tokens.push(Token::LeftBracket);
                chars.next();
            }
            ']' => {
                tokens.push(Token::RightBracket);
                chars.next();
            }
            ':' => {
                tokens.push(Token::Colon);
                chars.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            '"' => {
                let mut closed = false;
                chars.next();
                let mut collected = String::new();
                for nchar in chars.by_ref() {
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
                let mut num_string: String = String::new();
                while let Some(value) = chars.peek() {
                    if value.is_numeric()
                        || value.to_string().starts_with("-")
                        || value.to_string().contains(".")
                    {
                        num_string.push(*value);
                        chars.next();
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
                                position: 0,
                            });
                        }
                    }
                }
            }
            ch if ch == 't' || ch == 'f' || ch == 'n' => {
                let mut match_str = String::new();
                while let Some(nchar) = chars.peek() {
                    match nchar {
                        nchar if nchar.is_alphabetic() => {
                            match_str.push(*nchar);
                        }
                        _ => {
                            break;
                        }
                    }

                    if &match_str == "null" {
                        tokens.push(Token::Null);
                        match_str.clear();
                    } else if &match_str == "true" || &match_str == "false" {
                        let matched = match_str.parse();
                        match matched {
                            Ok(item) => tokens.push(Token::Boolean(item)),
                            Err(_error) => {
                                return Err(JsonError::UnexpectedToken {
                                    expected: "true or false boolean value".to_string(),
                                    found: match_str.to_string(),
                                    position: 0,
                                });
                            }
                        }
                        match_str.clear();
                    }
                    chars.next();
                }
            }
            ' ' | '\n' | '\r' | '\t' => {
                chars.next();
            }
            _ => {
                return Err(JsonError::UnexpectedToken {
                    expected: "valid JSON token".to_string(),
                    found: ch.to_string(),
                    position: 0,
                });
            }
        }
    }
    Ok(tokens)
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
        let tokens = tokenize("{}")?;
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::LeftBrace);
        assert_eq!(tokens[1], Token::RightBrace);
        Ok(())
    }
    #[test]
    fn test_simple_string() -> Result<()> {
        let tokens = tokenize(r#""hello""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("hello".to_string()));
        Ok(())
    }
    #[test]
    fn test_tokenize_string() -> Result<()> {
        let tokens = tokenize(r#""hello world""#)?;

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("hello world".to_string()));
        Ok(())
    }
    #[test]
    fn test_empty_string() -> Result<()> {
        let tokens = tokenize(r#""""#)?;

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_containing_json_special_chars() -> Result<()> {
        let tokens = tokenize(r#""{key: value}""#)?;

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("{key: value}".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_with_keyword_like_content() -> Result<()> {
        let tokens = tokenize(r#""not true or false""#)?;

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("not true or false".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_with_number_like_content() -> Result<()> {
        let tokens = tokenize(r#""phone: 555-1234""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("phone: 555-1234".to_string()));
        Ok(())
    }

    #[test]
    fn test_number() -> Result<()> {
        let tokens = tokenize("42")?;

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(42.0));
        Ok(())
    }

    #[test]
    fn test_negative_number() -> Result<()> {
        let tokens = tokenize("-42")?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(-42.0));
        Ok(())
    }

    #[test]
    fn test_decimal_number() -> Result<()> {
        let tokens = tokenize("0.5")?;

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(0.5));
        Ok(())
    }

    #[test]
    fn test_leading_decimal_number() -> Result<()> {
        let res = tokenize(".5");
        assert!(res.is_err());

        Ok(())
    }

    #[test]
    fn test_boolean_and_null() -> Result<()> {
        let tokens = tokenize("true false null")?;
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Boolean(true));
        assert_eq!(tokens[1], Token::Boolean(false));
        assert_eq!(tokens[2], Token::Null);
        Ok(())
    }

    #[test]
    fn test_simple_object() -> Result<()> {
        let tokens = tokenize(r#"{"name": "Alice"}"#)?;

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
        let tokens = tokenize(r#"{"age": 30, "active": true}"#)?;

        assert!(tokens.contains(&Token::String("age".to_string())));
        assert!(tokens.contains(&Token::Number(30.0)));
        assert!(tokens.contains(&Token::Comma));
        assert!(tokens.contains(&Token::String("active".to_string())));
        assert!(tokens.contains(&Token::Boolean(true)));
        Ok(())
    }

    #[test]
    fn test_json_with_brackets() -> Result<()> {
        let tokens = tokenize(r#"{"age": 30, "children_names": ["Naia", "Bryan"]}"#)?;

        assert_eq!(tokens.len(), 13);
        assert_eq!(tokens[7], Token::LeftBracket);
        assert_eq!(tokens[11], Token::RightBracket);
        Ok(())
    }
    #[test]
    fn test_unterminated_string_does_not_panic() -> Result<()> {
        let tokens = tokenize(r#""hello"#);
        assert!(tokens.is_err());
        Ok(())
    }
    #[test]
    fn test_leading_decimal_not_a_number() {
        // .5 is invalid JSON - numbers must have leading digit (0.5 is valid)
        let err = tokenize(".5").unwrap_err();
        assert!(matches!(
            err,
            JsonError::UnexpectedToken { position: 0, .. }
        ));
    }
    #[test]
    fn test_keyword_does_not_swallow_following_tokens() -> Result<()> {
        let tokens = tokenize("[true, null]")?;
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
        for input in ["nul", "tru", "fals", "txyz"] {
            assert!(
                matches!(tokenize(input), Err(JsonError::UnexpectedToken { .. })),
                "Should fail for: {}",
                input
            );
        }
    }

    #[test]
    fn test_malformed_number_is_error_not_panic() {
        for input in ["-", "1-2", "1.2.3", "--5"] {
            assert!(
                matches!(tokenize(input), Err(JsonError::InvalidNumber { .. })),
                "Should fail for: {}",
                input
            );
        }
    }

    #[test]
    fn test_non_ascii_digit_is_error_not_panic() {
        for input in ["½", "٣"] {
            assert!(tokenize(input).is_err(), "Should fail for: {}", input);
        }
    }
}
