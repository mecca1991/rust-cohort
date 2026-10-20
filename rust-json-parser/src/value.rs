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
                let final_val = val.replace("\"", "\\\"").replace("\n", "\\n");
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
                    if index == 0 {
                        write!(f, "{}:{}", k, v)?;
                    } else {
                        write!(f, ",{}:{}", k, v)?
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
    use crate::value::JsonValue;

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
}
