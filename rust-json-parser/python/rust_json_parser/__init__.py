"""Parse and format JSON using the Rust extension.

``parse_json`` reads JSON text, ``parse_json_file`` reads a UTF-8 file, and
``dumps`` formats Python values. Parsed numbers are floats. See each function's
docstring for accepted inputs, return values, examples, and exceptions.
"""

from ._rust_json_parser import parse_json, parse_json_file, dumps
