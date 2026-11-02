//! Python bindings enabled by the default `python` Cargo feature.
//!
//! PyO3 exposes `parse_json`, `parse_json_file`, and `dumps` through the
//! `_rust_json_parser` extension module. Parsed values become Python `None`,
//! `bool`, `float`, `str`, `list`, or `dict` objects. Parser errors become
//! `ValueError`; file-read errors use PyO3's conversion to `OSError` subclasses.
//! Python examples below are usage examples, not Rust doctests.

use pyo3::{IntoPyObjectExt, prelude::*};
use std::collections::HashMap;

use pyo3::{
    IntoPyObject, PyResult, Python,
    exceptions::{PyIOError, PyValueError},
    types::{PyAnyMethods, PyDict, PyList, PyListMethods},
};

use crate::{JsonError, JsonParser, JsonValue};

/// Recursively converts an owned JSON value to the corresponding Python object.
///
/// All JSON numbers become Python floats, including input written as integers.
/// Python container operations can propagate a `PyErr` during conversion.
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

/// Maps parsing errors to `ValueError` and file-related variants to `OSError`.
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
                "Invalid Unicode Sequence '{}' at position {}",
                sequence, position
            )),
            JsonError::IoError { message } => PyIOError::new_err(message),
            JsonError::FileNotFound { path } => {
                PyIOError::new_err(format!("File not found: {}", path))
            }
        }
    }
}

/// Parses a complete JSON string into Python values.
///
/// `input` is the JSON text. The result is a `dict`, `list`, `str`, `float`,
/// `bool`, or `None`, depending on the top-level value. Nested values use the
/// same conversions, and later duplicate object keys replace earlier values.
/// Every number becomes a `float`, so large integers may lose precision.
///
/// # Errors
///
/// Raises `ValueError` for text this parser rejects. That includes some valid
/// JSON: exponent notation such as `1e5` and surrogate-pair escapes such as
/// `"😀"`. Errors constructing Python objects are propagated.
///
/// Very deeply nested input overflows the stack and crashes the Python process
/// instead of raising an exception.
///
/// # Examples
///
/// ```python
/// from rust_json_parser import parse_json
///
/// value = parse_json('{"name": "Alice", "age": 30}')
/// assert value["name"] == "Alice"
/// assert value["age"] == 30.0
/// ```
#[pyfunction]
fn parse_json<'a>(py: Python<'a>, input: &'a str) -> PyResult<Bound<'a, PyAny>> {
    let parsed_data = JsonParser::new(input)?.parse()?;
    parsed_data.into_pyobject(py)
}

/// Reads a UTF-8 file at `path` and parses its complete contents into Python values.
///
/// Relative paths are resolved against the process's current working directory.
/// The whole file is loaded into memory before parsing, with the same value
/// conversions as `parse_json`.
///
/// # Errors
///
/// Raises an `OSError` subclass if the file cannot be read as UTF-8, including
/// `FileNotFoundError` for a missing file. Parsing failures raise `ValueError`;
/// Python object-conversion errors are propagated.
///
/// # Examples
///
/// This example requires an existing UTF-8 file containing JSON:
///
/// ```python
/// from rust_json_parser import parse_json_file
///
/// config = parse_json_file("config.json")
/// print(config)
/// ```
#[pyfunction]
fn parse_json_file<'a>(py: Python<'a>, path: &'a str) -> PyResult<Bound<'a, PyAny>> {
    let contents = std::fs::read_to_string(path)?; // std::io::Error → PyErr
    let result = JsonParser::new(&contents)?.parse()?; // JsonError → PyErr
    result.into_pyobject(py)
}

/// Serializes a supported Python value to compact or indented text.
///
/// `obj` may contain `None`, booleans, strings, lists, dictionaries, and numbers
/// that convert to `float`, such as `int`, `float`, and `Decimal`. Dictionary
/// keys are converted with Python `str()`; keys with the same string
/// representation can overwrite each other. Numbers pass through `float`, so
/// large integers may lose precision. Output key order is unspecified, and
/// forward slashes in strings are written as `\/`. Non-finite numbers are not
/// rejected: `float("nan")` is written as `NaN`, which is not valid JSON.
///
/// `indent=None` selects compact output. A nonnegative integer selects that
/// many spaces per nesting level; zero still adds line breaks to containers.
///
/// # Errors
///
/// Raises `ValueError` for unsupported values, such as tuples, sets, or an
/// `int` too large for a `float`. A negative `indent` raises `OverflowError`,
/// and a non-integer `indent` raises `TypeError`.
///
/// Cyclic containers are not detected: a list or dictionary that contains
/// itself crashes the Python process instead of raising an exception.
///
/// # Examples
///
/// ```python
/// from rust_json_parser import dumps
///
/// assert dumps([1, True, None]) == "[1,true,null]"
/// assert dumps([1, None], indent=2) == "[\n  1,\n  null\n]"
/// ```
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

/// Recursively converts supported Python objects into owned JSON values.
///
/// Boolean extraction precedes numeric extraction because Python booleans are
/// also integers. Dictionaries stringify their keys; unsupported values return
/// `ValueError`. Containers must be acyclic because visited objects are not tracked.
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

/// Registers the parsing and serialization functions on the Python extension.
///
/// Returns a Python error if adding a function to the module fails.
#[pymodule]
fn _rust_json_parser(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_json, m)?)?;
    m.add_function(wrap_pyfunction!(parse_json_file, m)?)?;
    m.add_function(wrap_pyfunction!(dumps, m)?)?;
    Ok(())
}
