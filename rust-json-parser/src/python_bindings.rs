//! Python bindings enabled by the default `python` Cargo feature.
//!
//! PyO3 exposes `parse_json`, `parse_json_file`, `dumps`, and
//! `benchmark_performance` through the `_rust_json_parser` extension module.
//! The benchmark compares parsing times with `json` and `simplejson`.
//! Parsed values become Python `None`, `bool`, `float`, `str`, `list`, or `dict`.
//! Parser errors become `ValueError`; file-read errors become `OSError` or a subclass.
//! Python examples below are usage examples, not Rust doctests.

use pyo3::{IntoPyObjectExt, prelude::*};
use std::collections::HashMap;
use std::time::Instant;

use pyo3::{
    exceptions::{PyTypeError, PyValueError},
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
                for item in arr {
                    items.append(item.into_pyobject(py)?)?;
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

/// Maps tokenization and parsing errors to Python's `ValueError`.
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
/// `"\ud83d\ude00"`. Literal emoji characters are accepted. Errors constructing
/// Python objects are propagated.
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
/// Raises `OSError` or one of its subclasses if the file cannot be read:
/// `FileNotFoundError` for a missing file, `IsADirectoryError` for a directory,
/// and plain `OSError` for content that is not valid UTF-8. Parsing failures
/// raise `ValueError`; Python object-conversion errors are propagated.
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
/// that convert to `float`, such as `int`, `float`, and `Decimal`. Numbers pass
/// through `float`, so large integers may lose precision.
///
/// Dictionary keys may be strings, `None`, booleans, or numbers convertible to
/// `float`. Strings are kept as-is; other keys use JSON scalar spellings, such
/// as `"null"` and `"true"`. Keys that convert to the same string overwrite earlier
/// values. Output key order is unspecified.
///
/// Forward slashes in strings are written as `\/`. Non-finite numbers are not
/// rejected: `float("nan")` is written as `NaN`, which is not valid JSON.
/// Control characters without a short JSON escape, such as NUL, are written
/// literally and also produce invalid JSON.
///
/// `indent=None` selects compact output. A nonnegative integer selects that
/// many spaces per nesting level; zero still adds line breaks to containers.
///
/// # Errors
///
/// Raises `TypeError` for unsupported values or dictionary keys, including
/// tuples, sets, and integers too large for a `float`. A negative or out-of-range
/// `indent` raises `OverflowError`; an incompatible type raises `TypeError`.
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

/// Keeps string keys unchanged and formats other supported keys as JSON scalars.
/// Containers and unsupported key types return `TypeError`.
fn get_dictionary_key(key: &Bound<PyAny>) -> PyResult<String> {
    if let Ok(k) = key.extract::<String>() {
        Ok(k)
    } else {
        let to_json_key = py_to_json_value(key)?;
        match to_json_key {
            JsonValue::Null => Ok("null".to_string()),
            JsonValue::Boolean(b) => Ok(b.to_string()),
            JsonValue::Number(n) => Ok(n.to_string()),
            _ => Err(PyTypeError::new_err(
                "Invalid dictionary key type for JSON conversion. Only strings, numbers, booleans, and null are allowed.",
            )),
        }
    }
}
/// Recursively converts supported Python objects into owned JSON values.
///
/// Unsupported values return `TypeError`. Containers must be acyclic because
/// visited objects are not tracked.
fn py_to_json_value(obj: &Bound<PyAny>) -> PyResult<JsonValue> {
    if obj.is_none() {
        return Ok(JsonValue::Null);
    }
    // Check booleans before numbers because Python bool is a subclass of int.
    if let Ok(b) = obj.extract::<bool>() {
        return Ok(JsonValue::Boolean(b));
    }
    if let Ok(n) = obj.extract::<f64>() {
        return Ok(JsonValue::Number(n));
    }
    if let Ok(s) = obj.extract::<String>() {
        return Ok(JsonValue::String(s));
    }
    if let Ok(list) = obj.cast::<PyList>() {
        let mut arr = Vec::new();
        for item in list.iter() {
            arr.push(py_to_json_value(&item)?);
        }
        return Ok(JsonValue::Array(arr));
    }
    if let Ok(dict) = obj.cast::<PyDict>() {
        let mut dict_items = HashMap::new();
        for (key, value) in dict {
            let key_str = get_dictionary_key(&key)?;
            dict_items.insert(key_str, py_to_json_value(&value)?);
        }
        return Ok(JsonValue::Object(dict_items));
    }
    Err(PyTypeError::new_err("Unsupported type for JSON conversion"))
}

/// Measures repeated parsing with this parser, Python's `json`, and `simplejson`.
///
/// # Arguments
///
/// `json_str` is raw JSON text accepted by all three parsers. Input limitations
/// from `parse_json` apply. `iterations` defaults to 1000 and must be a
/// nonnegative integer that fits in `usize`. Zero skips parsing but still imports
/// the Python modules. The `simplejson` package must be installed.
///
/// # Returns
///
/// A tuple `(rust_time, json_time, simplejson_time)` of total elapsed seconds as
/// Python floats. Each value measures the whole loop, not an average per parse.
///
/// # Timing
///
/// Imports and function lookup happen before timing. The loops run sequentially
/// in tuple order without warm-up runs. Rust timings include conversion to Python
/// objects; the other loops call each module's `loads` function. Installed module
/// defaults determine the backends. Serialization and printing are not timed.
///
/// # Errors
///
/// Import, parsing, and Python object-conversion errors propagate immediately;
/// no partial timings are returned. A missing `simplejson` raises
/// `ModuleNotFoundError`. Negative or out-of-range iterations raise `OverflowError`;
/// incompatible argument types raise `TypeError`.
///
/// # Examples
///
/// ```python
/// from rust_json_parser import benchmark_performance
///
/// rust_time, json_time, simplejson_time = benchmark_performance(
///     '{"items": [1, true, null]}', iterations=10_000
/// )
/// print(f"Rust total: {rust_time:.6f}s")
/// print(f"json total: {json_time:.6f}s")
/// print(f"simplejson total: {simplejson_time:.6f}s")
/// ```
#[pyfunction]
#[pyo3(signature = (json_str, iterations=1000))]
fn benchmark_performance(
    py: Python<'_>,
    json_str: &str,
    iterations: usize,
) -> PyResult<(f64, f64, f64)> {
    // Resolve both Python callables before measuring the parsing loops.
    let json_module = py.import("json")?;
    let json_loads = json_module.getattr("loads")?;

    let simplejson_module = py.import("simplejson")?;
    let simplejson_loads = simplejson_module.getattr("loads")?;

    // Include conversion of the parsed Rust value into Python objects.
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = parse_json(py, json_str)?;
    }
    let rust_duration = start.elapsed();

    let start_json_module = Instant::now();
    for _ in 0..iterations {
        let _ = json_loads.call1((json_str,))?;
    }
    let json_module_duration = start_json_module.elapsed();

    let start_simple_json = Instant::now();
    for _ in 0..iterations {
        let _ = simplejson_loads.call1((json_str,))?;
    }
    let simple_json_duration = start_simple_json.elapsed();

    Ok((
        rust_duration.as_secs_f64() as f64,
        json_module_duration.as_secs_f64() as f64,
        simple_json_duration.as_secs_f64() as f64,
    ))
}

/// Registers parsing, serialization, and benchmarking on the Python extension.
///
/// Returns a Python error if adding a function to the module fails.
#[pymodule]
fn _rust_json_parser(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_json, m)?)?;
    m.add_function(wrap_pyfunction!(parse_json_file, m)?)?;
    m.add_function(wrap_pyfunction!(dumps, m)?)?;
    m.add_function(wrap_pyfunction!(benchmark_performance, m)?)?;
    Ok(())
}
