use crate::error::JsonError;
use crate::tokenizer::{Token, tokenize};
use crate::value::JsonValue;

type Result<T> = std::result::Result<T, JsonError>;

pub fn parse_json(input: &str) -> Result<JsonValue> {
    let input = input.trim();
    let tokens = tokenize(input)?;
    let token = tokens.get(0);

    if let Some(token) = token {
        match token {
            Token::Boolean(value) => Ok(JsonValue::Boolean(*value)),
            Token::String(value) => {
                if value.is_empty() {
                    Err(JsonError::UnexpectedEndOfInput {
                        expected: "JSON value".to_string(),
                        position: 0,
                    })
                } else {
                    let cloned_val = value.clone();
                    Ok(JsonValue::String(cloned_val))
                }
            }
            Token::Number(value) => Ok(JsonValue::Number(*value)),
            Token::Null => Ok(JsonValue::Null),
            _ => Err(JsonError::UnexpectedEndOfInput {
                expected: "JSON value".to_string(),
                position: 0,
            }),
        }
    } else {
        Err(JsonError::UnexpectedEndOfInput {
            expected: "JSON value".to_string(),
            position: 0,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    type Result<T> = std::result::Result<T, JsonError>;

    #[test]
    fn test_parse_string() -> Result<()> {
        let result = parse_json(r#""hello world""#)?;
        assert_eq!(result, JsonValue::String("hello world".to_string()));
        Ok(())
    }

    #[test]
    fn test_parse_number() -> Result<()> {
        let result = parse_json("42.5")?;
        assert_eq!(result, JsonValue::Number(42.5));

        let result = parse_json("0")?;
        assert_eq!(result, JsonValue::Number(0.0));

        let result = parse_json("-10")?;
        assert_eq!(result, JsonValue::Number(-10.0));
        Ok(())
    }

    #[test]
    fn test_parse_boolean() -> Result<()> {
        let result = parse_json("true")?;
        assert_eq!(result, JsonValue::Boolean(true));

        let result = parse_json("false")?;
        assert_eq!(result, JsonValue::Boolean(false));
        Ok(())
    }

    #[test]
    fn test_parse_null() -> Result<()> {
        let result = parse_json("null")?;
        assert_eq!(result, JsonValue::Null);
        Ok(())
    }

    #[test]
    fn test_parse_error_empty() {
        let result = parse_json("");
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
        let result = parse_json("@");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_with_whitespace() -> Result<()> {
        let result = parse_json("  42  ")?;
        assert_eq!(result, JsonValue::Number(42.0));

        let result = parse_json("\n\ttrue\n")?;
        assert_eq!(result, JsonValue::Boolean(true));
        Ok(())
    }

    #[test]
    fn test_result_pattern_matching() {
        let result = parse_json("42");

        match result {
            Ok(JsonValue::Number(n)) => assert_eq!(n, 42.0),
            _ => panic!("Expected successful number parse"),
        }

        let result = parse_json("@invalid@");

        match result {
            Err(JsonError::UnexpectedToken { .. }) => {} // Expected
            _ => panic!("Expected UnexpectedToken error"),
        }
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
    fn test_unterminated_string() {
        let err = tokenize(r#""missing end quote"#).unwrap_err();
        match err {
            JsonError::UnexpectedEndOfInput { position, .. } => {
                assert_eq!(position, 0);
            }
            other => panic!("expected UnexpectedEndOfInput, got {:?}", other),
        }
    }
}
