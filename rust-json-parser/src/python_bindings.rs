use pyo3::{IntoPyObjectExt, prelude::*};
use std::collections::HashMap;

use pyo3::{
    IntoPyObject, PyResult, Python,
    exceptions::{PyIOError, PyValueError},
    types::{PyAnyMethods, PyDict, PyList, PyListMethods},
};

use crate::{JsonError, JsonParser, JsonValue};

impl<'py> IntoPyObject<'py> for JsonValue {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            JsonValue::Null => Ok(py.None().into_bound(py)),
            JsonValue::Boolean(b) => Ok(b.into_bound_py_any(py)?),
            JsonValue::Number(n) => Ok(n.into_bound_py_any(py)?),
            JsonValue::String(s) => Ok(s.into_bound_py_any(py)?),
            JsonValue::Array(arr) => {
                let items = PyList::empty(py);
                for item in arr.iter() {
                    items.append(item.to_owned().into_pyobject(py)?)?;
                }
                Ok(items.into_any())
            }
            JsonValue::Object(obj) => {
                let dict_items = PyDict::new(py);
                for (key, value) in obj {
                    dict_items.set_item(key, value.into_pyobject(py)?)?;
                }
                Ok(dict_items.into_any())
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
                "Invalide Unicode Sequence '{}' at position {}",
                sequence, position
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
fn parse_json<'a>(py: Python<'a>, input: &'a str) -> PyResult<Bound<'a, PyAny>> {
    let parsed_data = JsonParser::new(input)?.parse()?;
    parsed_data.into_pyobject(py)
}

#[pyfunction]
fn parse_json_file<'a>(py: Python<'a>, path: &'a str) -> PyResult<Bound<'a, PyAny>> {
    let contents = std::fs::read_to_string(path)?; // std::io::Error → PyErr
    let result = JsonParser::new(&contents)?.parse()?; // JsonError → PyErr
    result.into_pyobject(py)
}

#[pyfunction]
#[pyo3(signature = (obj, indent=None))]
fn dumps(obj: &Bound<PyAny>, indent: Option<usize>) -> PyResult<String> {
    let converted = py_to_json_value(obj)?;
    match indent {
        Some(size) => {
            let val = converted.pretty_print(size);
            Ok(val)
        }
        None => Ok(converted.to_string()),
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
        for item in list.iter() {
            arr.push(py_to_json_value(&item)?);
        }
        return Ok(JsonValue::Array(arr));
    }
    // 6. Check dict (recurse on values)
    if let Ok(dict) = obj.cast::<PyDict>() {
        let mut dict_items = HashMap::new();
        for (key, value) in dict {
            let k = key.to_string();
            let converted_value = py_to_json_value(&value)?;
            dict_items.insert(k, converted_value);
        }
        return Ok(JsonValue::Object(dict_items));
    }

    // 7. Unsupported type
    Err(PyValueError::new_err(
        "Unsupported type for JSON conversion",
    ))
}

// Module registration
#[pymodule]
fn _rust_json_parser(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_json, m)?)?;
    m.add_function(wrap_pyfunction!(parse_json_file, m)?)?;
    m.add_function(wrap_pyfunction!(dumps, m)?)?;
    Ok(())
}
