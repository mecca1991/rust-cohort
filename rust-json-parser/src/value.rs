use std::{collections::HashMap, fmt};

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

impl JsonValue {
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(msg) => Some(msg),
            _ => None,
        }
    }
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            JsonValue::Number(val) => Some(*val),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Boolean(val) => Some(*val),
            _ => None,
        }
    }
    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        match self {
            JsonValue::Array(val) => Some(val),
            _ => None,
        }
    }
    pub fn as_object(&self) -> Option<&HashMap<String, JsonValue>> {
        match self {
            JsonValue::Object(val) => Some(val),
            _ => None,
        }
    }
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(obj) => obj.get(key),
            _ => None,
        }
    }
    pub fn get_index(&self, index: usize) -> Option<&JsonValue> {
        match self {
            JsonValue::Array(arr) => arr.get(index),
            _ => None,
        }
    }

    pub fn pretty_print(&self, indent: usize) -> String {
        let depth = 0;
        self.pretty_print_recurse(indent, depth)
    }

    fn pretty_print_recurse(&self, indent: usize, depth: usize) -> String {
        let indent_spaces = " ".repeat(indent * (depth + 1));
        let closing_spaces = " ".repeat(indent * depth);

        let mut final_items: Vec<String> = Vec::new();
        match self {
            JsonValue::Object(items) => {
                if items.is_empty() {
                    return "{}".to_string();
                }
                for (key, val) in items {
                    let key = JsonValue::String(key.clone());
                    let value = val.pretty_print_recurse(indent, depth + 1);
                    final_items.push(format!("{}{}: {}", indent_spaces, key, value));
                }
                format!("{{\n{}\n{}}}", final_items.join(",\n"), closing_spaces)
            }
            JsonValue::Array(items) => {
                if items.is_empty() {
                    return "[]".to_string();
                }
                for item in items {
                    let value = item.pretty_print_recurse(indent, depth + 1);
                    final_items.push(format!("{}{}", indent_spaces, value));
                }
                format!("[\n{}\n{}]", final_items.join(",\n"), closing_spaces)
            }
            _ => self.to_string(),
        }
    }
}

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonValue::Null => {
                write!(f, "null")
            }
            JsonValue::Number(num) => {
                write!(f, "{}", num)
            }
            JsonValue::Boolean(val) => {
                write!(f, "{}", val)
            }
            JsonValue::String(val) => {
                let final_val = val
                    .replace("\\", "\\\\")
                    .replace("\"", "\\\"")
                    .replace("\n", "\\n")
                    .replace("/", "\\/")
                    .replace("\x08", "\\b")
                    .replace("\x0C", "\\f")
                    .replace("\r", "\\r")
                    .replace("\t", "\\t");
                write!(f, "\"{}\"", final_val)
            }
            JsonValue::Array(items) => {
                write!(f, "[")?;
                for (index, value) in items.iter().enumerate() {
                    if index == 0 {
                        write!(f, "{}", value)?;
                    } else {
                        write!(f, ",{}", value)?
                    }
                }
                write!(f, "]")?;
                Ok(())
            }
            JsonValue::Object(items) => {
                write!(f, "{{")?;
                for (index, (k, v)) in items.iter().enumerate() {
                    let key = JsonValue::String(k.clone());
                    if index == 0 {
                        write!(f, "{}:{}", key, v)?;
                    } else {
                        write!(f, ",{}:{}", key, v)?
                    }
                }
                write!(f, "}}")?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{JsonError, JsonParser, value::JsonValue};
    use std::collections::HashMap;

    #[test]
    fn test_json_value_equality() {
        assert_eq!(JsonValue::Null, JsonValue::Null);
        assert_eq!(JsonValue::Boolean(true), JsonValue::Boolean(true));
        assert_eq!(JsonValue::Number(42.0), JsonValue::Number(42.0));

        assert_eq!(
            JsonValue::String("test".to_string()),
            JsonValue::String("test".to_string())
        );
    }

    #[test]
    fn test_json_value_creation() {
        let null_val = JsonValue::Null;
        let bool_val = JsonValue::Boolean(true);
        let num_val = JsonValue::Number(42.5);
        let str_val = JsonValue::String("hello".to_string());

        assert!(null_val.is_null());
        assert_eq!(bool_val.as_bool(), Some(true));
        assert_eq!(num_val.as_f64(), Some(42.5));
        assert_eq!(str_val.as_str(), Some("hello"));
    }

    #[test]
    fn test_json_value_accessors() {
        let value = JsonValue::String("test".to_string());
        assert_eq!(value.as_str(), Some("test"));
        assert_eq!(value.as_f64(), None);
        assert_eq!(value.as_bool(), None);
        assert!(!value.is_null());

        let value = JsonValue::Number(42.0);
        assert_eq!(value.as_f64(), Some(42.0));
        assert_eq!(value.as_str(), None);

        let value = JsonValue::Boolean(true);
        assert_eq!(value.as_bool(), Some(true));

        let value = JsonValue::Null;
        assert!(value.is_null());
    }

    #[test]
    fn test_display_primitives() {
        assert_eq!(JsonValue::Null.to_string(), "null");
        assert_eq!(JsonValue::Boolean(true).to_string(), "true");
        assert_eq!(JsonValue::Boolean(false).to_string(), "false");
        assert_eq!(JsonValue::Number(42.0).to_string(), "42");
        assert_eq!(JsonValue::Number(3.14).to_string(), "3.14");
        assert_eq!(
            JsonValue::String("hello".to_string()).to_string(),
            "\"hello\""
        );
    }

    #[test]
    fn test_display_array() {
        let value = JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)]);
        assert_eq!(value.to_string(), "[1,2]");
    }

    #[test]
    fn test_display_empty_containers() {
        assert_eq!(JsonValue::Array(vec![]).to_string(), "[]");
        assert_eq!(JsonValue::Object(HashMap::new()).to_string(), "{}");
    }

    #[test]
    fn test_display_escape_string() {
        let value = JsonValue::String("hello\nworld".to_string());
        assert_eq!(value.to_string(), "\"hello\\nworld\"");
    }

    #[test]
    fn test_display_escape_quotes() {
        let value = JsonValue::String("say \"hi\"".to_string());
        assert_eq!(value.to_string(), "\"say \\\"hi\\\"\"");
    }

    #[test]
    fn test_display_nested() -> Result<(), JsonError> {
        let mut parser = JsonParser::new(r#"{"arr": [1, 2]}"#)?;
        let value = parser.parse()?;
        let output = value.to_string();
        // Object key order may vary, so check components
        assert!(output.contains("\"arr\""));
        assert!(output.contains("[1,2]"));
        Ok(())
    }

    #[test]
    fn test_display_nested_array() {
        let value = JsonValue::Array(vec![JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
        ])]);
        assert_eq!(value.to_string(), "[[1,2]]");
    }
}
