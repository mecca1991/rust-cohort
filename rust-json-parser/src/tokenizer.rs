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
    Null
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, JsonError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut chars = input.chars().peekable();
    let mut token_position = 0;
    while let Some(&ch) = chars.peek() {
        match ch {
            '{' => {
                tokens.push(Token::LeftBrace);
                chars.next();
                token_position += 1;
            }
            '}' => {
                tokens.push(Token::RightBrace);
                chars.next();
                token_position += 1;
            }
            '[' => {
                tokens.push(Token::LeftBracket);
                chars.next();
                token_position += 1;
            }
            ']' => {
                tokens.push(Token::RightBracket);
                chars.next();
                token_position += 1;
            }
            ':' => {
                tokens.push(Token::Colon);
                chars.next();
                token_position += 1;
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
                token_position += 1;
            }
            '"'  => {
                chars.next();
                let mut collected = String::new();
                for nchar in chars.by_ref() {
                    if nchar == '"' {
                        break;
                    }
                    collected.push(nchar);
                    token_position += 1;
                }
                tokens.push(Token::String(collected));
            }
            ch if ch.is_numeric() || (ch == '-') || (ch == '.') => {
                let mut num_string: String = String::new();
                while let Some(value) = chars.peek() {
                    if (value.is_numeric() || ['-', '.'].contains(value)) && !num_string.starts_with(".") {
                        num_string.push(*value);
                        chars.next();
                        token_position += 1;
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
                    tokens.push(Token::Number(
                        num_string.parse::<f64>().expect("Invalid Number!"),
                    ));
                }
            }
            ch if ch == 't' || ch == 'f' || ch == 'n' => {
                let mut match_str = String::new();
                let boolean_values = ["true".to_string(), "false".to_string()];
                while let Some(nchar) = chars.peek() {
                    if nchar.is_alphabetic() && !boolean_values.contains(&match_str) {
                        match_str.push(*nchar);
                    }
                    if &match_str == "null" {
                        tokens.push(Token::Null);
                        match_str.clear();
                    }
                    if boolean_values.contains(&match_str) {
                        tokens.push(Token::Boolean(
                            match_str.parse().expect("Invalid Boolean value"),
                        ));
                        match_str.clear();
                    }
                    chars.next();
                    token_position += 1;
                }
            }
            ' ' => {
                chars.next();
                token_position += 1;
            }
            _ => {
                return Err(JsonError::UnexpectedToken { 
                    expected: "JSON value".to_string(), 
                    found: ch.to_string(),
                    position: token_position
                })
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
    fn test_empty_string() -> Result<()> {
        // Outer boundary: adjacent quotes with no inner content
        let tokens = tokenize(r#""""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_containing_json_special_chars() -> Result<()> {
        // Inner handling: JSON delimiters inside strings don't break tokenization
        let tokens = tokenize(r#""{key: value}""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("{key: value}".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_with_keyword_like_content() -> Result<()> {
        // Inner handling: "true", "false", "null" inside strings stay as string content
        let tokens = tokenize(r#""not true or false""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("not true or false".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_with_number_like_content() -> Result<()> {
        // Inner handling: numeric content inside strings doesn't become Number tokens
        let tokens = tokenize(r#""phone: 555-1234""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("phone: 555-1234".to_string()));
        Ok(())
    }

    // Number parsing tests

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
}