"""Pretty-print JSON supplied as text, a file path, or standard input.

Run ``python -m rust_json_parser '{"name": "Alice"}'`` after installing the
extension. An existing file passed as the first argument is read as UTF-8;
otherwise that argument is parsed as JSON text. With no arguments, all of
standard input is read. Output uses two-space indentation. Parsing and I/O
exceptions propagate to the caller.
"""

import sys
import os.path
import rust_json_parser as rjp

if __name__ == "__main__":
    if len(sys.argv) > 1:
        user_input = sys.argv[1]
        if os.path.isfile(user_input):
            result = rjp.parse_json_file(user_input)
        else:
            result = rjp.parse_json(user_input)
    else:
        result = rjp.parse_json(sys.stdin.read())

    print(rjp.dumps(result, indent=2))
