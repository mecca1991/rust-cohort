# Rust cohort: JSON parser

A learning project that parses JSON into Rust values and exposes parsing and
formatting functions to Python through PyO3. The crate lives in
[`rust-json-parser/`](rust-json-parser/).

## Rust usage

The crate uses Rust's 2024 edition. Run Cargo commands from the crate directory.
`--no-default-features` turns off the Python bindings, so Cargo does not need a
Python interpreter:

```sh
cd rust-json-parser
cargo run --no-default-features
```

The binary demonstrates parsing a fixed string. Library callers use `JsonParser`:

```rust
use rust_json_parser::{JsonParser, JsonValue};

fn main() -> rust_json_parser::Result<()> {
    let value = JsonParser::new(r#"{"name": "Alice", "active": true}"#)?.parse()?;
    assert_eq!(value.get("name").and_then(JsonValue::as_str), Some("Alice"));
    println!("{}", value.pretty_print(2));
    Ok(())
}
```

`JsonParser::new` tokenizes the input; `parse` checks the document structure and
rejects trailing tokens. `parse_value` reads just the next value. Parsing advances
the cursor, so create a new parser to read the original input again.

## Documentation and tests

Documentation follows the cohort's
[Documentation in Rust guide](https://github.com/bbelderbos/rust-cohort-sept-2026/blob/main/week6/07-documentation-in-rust.md):
`//!` describes each module, `///` describes items, and Rust examples are executable
doctests. Fallible public methods describe their errors in `# Errors` sections.

From `rust-json-parser/`:

```sh
# Run unit tests and documentation examples without Python.
cargo test --no-default-features

# Run only the documentation examples.
cargo test --doc --no-default-features

# Generate and open the public Rust API documentation.
cargo doc --no-deps --no-default-features --open

# Include internal modules and Python bindings (needs a Python 3 interpreter).
cargo doc --no-deps --document-private-items

# Check formatting.
cargo fmt --check
```

The public documentation starts at `target/doc/rust_json_parser/index.html`.
Without `--no-default-features`, Cargo also compiles the PyO3 bindings, which
needs a Python 3 interpreter (set `PYO3_PYTHON` to choose one). Python examples
in doc comments are not run by Rust's doctest runner.

## Python usage

The Python package requires Python 3.12 or newer. With Rust and Maturin
installed, build and install the extension into an activated virtual
environment, then run the Python tests:

```sh
cd rust-json-parser
maturin develop
pytest tests/
```

```python
from rust_json_parser import dumps, parse_json, parse_json_file

value = parse_json('{"name": "Alice", "age": 30}')
assert value["age"] == 30.0
print(dumps(value, indent=2))

# Requires an existing UTF-8 JSON file:
# config = parse_json_file("config.json")
```

Parsed values become Python `None`, `bool`, `float`, `str`, `list`, or `dict`.
Parsing failures raise `ValueError`; file-read failures raise an `OSError`
subclass. `dumps` supports these same types plus integers within `f64` range,
converts dictionary keys to strings, and raises `ValueError` for unsupported
types such as tuples. Its `indent` argument defaults to `None` for compact
output; `indent=0` adds line breaks without spaces. Forward slashes in strings
are written as `\/`.

After installation, the Python CLI accepts JSON text, an existing file path, or
standard input and prints with two-space indentation:

```sh
python -m rust_json_parser '{"name": "Alice"}'
python -m rust_json_parser config.json
```

## Source layout

| File | Responsibility |
| --- | --- |
| `src/lib.rs` | Crate overview, public exports, and the `Result` alias |
| `src/tokenizer.rs` | Character scanning, escape decoding, and tokens |
| `src/parser.rs` | Stateful parsing of tokens into nested values |
| `src/value.rs` | JSON values, typed accessors, and formatting |
| `src/error.rs` | Error variants, diagnostic fields, and display messages |
| `src/python_bindings.rs` | Python conversions and exported functions |
| `src/main.rs` | Fixed-input Rust demonstration |
| `python/rust_json_parser/` | Python imports and command-line entry point |
| `tests/test_python_integration.py` | Python integration tests |

Paths in this table are relative to `rust-json-parser/`.

## Current limitations

- Numbers use `f64`, so large integers may lose precision. Exponent notation is
  unsupported, and validation accepts some nonstandard forms such as leading zeroes.
- Unicode escapes are decoded individually; surrogate-pair escapes are unsupported.
- Strings may contain unescaped control characters, such as raw newlines, which
  standard JSON rejects.
- Objects have unspecified key order, and the last duplicate key wins.
- Tokenization errors report character indices; structural errors report token
  indices. Some tokenizer errors report zero or the cursor after the invalid text.
- Formatting does not reject non-finite numbers or escape every control character,
  so output can be invalid JSON, such as `NaN`.
- Parsing, formatting, and Python conversion recurse without a depth limit, and
  Python conversion does not detect cyclic containers. Deeply nested input or a
  list that contains itself overflows the stack and crashes the process instead
  of returning an error; in Python, this kills the interpreter.
