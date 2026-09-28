mod error;
mod parser;
mod tokenizer;
mod value;

pub use error::JsonError;
pub use parser::parse_json;
pub use tokenizer::{Token, tokenize};
pub use value::JsonValue;

#[cfg(test)]
mod tests {
    
    use crate::error::JsonError;
    use crate::tokenizer::Token;
    use crate::tokenizer::tokenize;

    #[test]
    fn test_empty_braes() -> Result<(), JsonError> {
        let tokens = tokenize("{}")?;
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::LeftBrace);
        assert_eq!(tokens[1], Token::RightBrace);
        Ok(())

    }
    #[test]
    fn test_simple_string() -> Result<(), JsonError> {
        let tokens = tokenize(r#""hello""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("hello".to_string()));
        Ok(())
    }
    #[test]
    fn test_tokenize_string() -> Result<(), JsonError>  {
        let tokens = tokenize(r#""hello world""#)?;

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("hello world".to_string()));
        Ok(())
    }
    #[test]
    fn test_empty_string() -> Result<(), JsonError>  {
        let tokens = tokenize(r#""""#)?;

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_containing_json_special_chars() -> Result<(), JsonError> {
        let tokens = tokenize(r#""{key: value}""#)?;

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("{key: value}".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_with_keyword_like_content()-> Result<(), JsonError> {
        let tokens = tokenize(r#""not true or false""#)?;

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("not true or false".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_with_number_like_content() -> Result<(), JsonError>  {
        let tokens = tokenize(r#""phone: 555-1234""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("phone: 555-1234".to_string()));
        Ok(())
    }

    #[test]
    fn test_number() -> Result<(), JsonError>  {
        let tokens = tokenize("42")?;

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(42.0));
        Ok(())
    }

    #[test]
    fn test_negative_number() -> Result<(), JsonError> {
        let tokens = tokenize("-42")?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(-42.0));
        Ok(())
    }

    #[test]
    fn test_decimal_number() -> Result<(), JsonError> {
        let tokens = tokenize("0.5")?;

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(0.5));
        Ok(())
    }

    #[test]
    fn test_leading_decimal_number() -> Result<(), JsonError>  {
        let tokens = tokenize(".5")?;
        assert!(!tokens.contains(&Token::Number(0.5)));
        Ok(())
    }

    #[test]
    fn test_boolean_and_null() -> Result<(), JsonError> {
        let tokens = tokenize("true false null")?;
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Boolean(true));
        assert_eq!(tokens[1], Token::Boolean(false));
        assert_eq!(tokens[2], Token::Null);
        Ok(())
    }

    #[test]
    fn test_simple_object() -> Result<(), JsonError> {
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
    fn test_multiple_values() -> Result<(), JsonError> {
        let tokens = tokenize(r#"{"age": 30, "active": true}"#)?;

        assert!(tokens.contains(&Token::String("age".to_string())));
        assert!(tokens.contains(&Token::Number(30.0)));
        assert!(tokens.contains(&Token::Comma));
        assert!(tokens.contains(&Token::String("active".to_string())));
        assert!(tokens.contains(&Token::Boolean(true)));
        Ok(())
    }

    #[test]
    fn test_json_with_brackets() -> Result<(), JsonError>  {
        let tokens = tokenize(r#"{"age": 30, "children_names": ["Naia", "Bryan"]}"#)?;

        assert_eq!(tokens.len(), 13);
        assert_eq!(tokens[7], Token::LeftBracket);
        assert_eq!(tokens[11], Token::RightBracket);
        Ok(())
    }
    #[test]
    fn test_unterminated_string_does_not_panic() -> Result<(), JsonError> {
        let tokens = tokenize(r#""hello"#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("hello".to_string()));
        Ok(())
    }
}
