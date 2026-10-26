use std::{collections::HashMap};

use pyo3::{
    IntoPyObject, IntoPyObjectExt,
    exceptions::PyValueError,
    types::{PyAnyMethods, PyDict},
};

use crate::{JsonError, JsonParser, JsonValue};

impl<'py> IntoPyObject<'py> for JsonValue {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            JsonValue::Null => Ok(py.None()),
            JsonValue::Boolean(b) => Ok(b.into_pyobject(py)?),
            JsonValue::Number(n) => Ok(n.into_pyobject(py)?),
            JsonValue::String(s) => Ok(s.into_pyobject(py)?),
            JsonValue::Array(arr) => {
                let py_list = PyList::empty(py);
                for item in arr {
                    py_list.append(item.into_pyobject(py)?)?;
                }
                Ok(py_list)
            }
            JsonValue::Object(obj) => {
                let dict_items = PyDict::new(py);
                for (key, value) in obj {
                    dict_items.set_item(key, value.into_pyobject(py)?)?;
                }
                Ok(dict_items)
            }
        }
    }
}

impl From<JsonError> for PyErr {
    fn from(err: JsonError) -> PyErr {
        match err {
            JsonError::UnexpectedToken {
                expected,
                found,
                position,
            } => PyValueError::new_err(format!(
                "Unexpected token at position {}: expected {}, found {}",
                position, expected, found
            )),
            JsonError::UnexpectedEndOfInput { expected, position } => {
                PyValueError::new_err(format!(
                    "Unexpected end of input at position {}: expected {}",
                    position, expected
                ))
            }
            JsonError::InvalidEscape { char, position } => PyValueError::new_err(format!(
                "Invalid escape character '{}' at position {}",
                char, position
            )),
            JsonError::InvalidNumber { value, position } => PyValueError::new_err(format!(
                "Invalid Number provided '{}' at position {}",
                value, position
            )),
            JsonError::InvalidUnicode { sequence, position } => PyValueError::new_err(format!(
                "Invalide Unicode Sequence '{}' at position {}", sequence, position
            )),
            JsonError::IoError { message } => PyIOError::new_err(message),
            JsonError::FileNotFound { path } => {
                PyIOError::new_err(format!("File not found: {}", path))
            } 
        }
    }
}

// Python-callable functions
#[pyfunction]
fn parse_json(py: Python<'_>, input: &str) -> PyResult<Bound<'_, PyAny>> {
    let parsed_data = JsonParser::new(input)?.parse()?;
    parsed_data.into_pyobject(py)
}

#[pyfunction]
fn parse_json_file(py: Python<'_>, path: &str) -> PyResult<Bound<'_, PyAny>> {
    let contents = std::fs::read_to_string(path)?;  // std::io::Error → PyErr
    let result = JsonParser::new(&contents)?.parse();      // JsonError → PyErr
    result.into_pyobject(py)
}

#[pyfunction]
#[pyo3(signature = (obj, indent=None))]
fn dumps(obj: &Bound<PyAny>, indent: Option<usize>) -> PyResult<String> {
    let converted = py_to_json_value(obj)?;
    match indent {
        Some(ind) => {
            Ok(converted.fmt())
        },
        None => {
            Ok(converted.to_string())
        }
    }
}

// Helper (not exposed to Python)
fn py_to_json_value(obj: &Bound<PyAny>) -> PyResult<JsonValue> {
    // 1. Check None first
    if obj.is_none() {
        return Ok(JsonValue::Null);
    }
    // 2. Check bool before numbers (critical!)
    if let Ok(b) = obj.extract::<bool>() {
        return Ok(JsonValue::Boolean(b));
    }
    // 3. Check numbers
    if let Ok(n) = obj.extract::<f64>() {
        return Ok(JsonValue::Number(n));
    }
    // 4. Check string
    if let Ok(s) = obj.extract::<String>() {
        return Ok(JsonValue::String(s));
    }
    // 5. Check list (recurse on elements)
    if let Ok(list) = obj.cast::<PyList>() {
       // Converting a Python list
        let mut arr = Vec::new();
        for item in list {
            arr.push(py_to_json_value(&item)?);
        }
        return Ok(JsonValue::Array(arr));
    }
    // 6. Check dict (recurse on values)
    if let Ok(dict) = obj.cast::<PyDict>() {
        let mut dict_items= HashMap::new();
        for (key, value) in dict {
            let converted_value = py_to_json_value(&value)?;
            dict_items.insert(key, converted_value);
        }
        return Ok(JsonValue::Object(dict_items));
    }

    // 7. Unsupported type
    Err(PyValueError::new_err("Unsupported type for JSON conversion"))
}

// Module registration
#[pymodule]
fn _rust_json_parser(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_json, m)?)?;
    m.add_function(wrap_pyfunction!(parse_json_file, m)?)?;
    m.add_function(wrap_pyfunction!(dumps, m)?)?;
    Ok(())
}
