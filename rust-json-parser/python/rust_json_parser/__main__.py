"""Pretty-print JSON or benchmark a fixed sample with three parsers.

Run ``python -m rust_json_parser '{"name": "Alice"}'`` after installing the
extension. An existing file passed as the input argument is read as UTF-8;
otherwise the argument is parsed as JSON text. With no input argument, piped
standard input is read; an interactive terminal shows usage and exits with
status 1. Explicit input takes precedence over standard input. Output uses
two-space indentation. Use ``-h`` or ``--help`` to display the available options.
Arguments are handled directly through ``sys.argv``; at most one input or
option may be supplied.

Parsing and I/O errors are written to standard error with exit status 1;
invalid command-line arguments exit with status 2. Parse failures and files
that are not valid UTF-8 are reported as invalid JSON; other read failures are
reported as file errors.

Run ``python -m rust_json_parser --benchmark`` without an input argument to
time 1,000 parses of the fixed sample ``{"key": "value"}`` with this parser,
Python's ``json`` module, and ``simplejson``. Timings are totals in seconds.
Benchmark mode ignores standard input and reports whether Rust is faster,
slower, or equally fast.
"""

import os.path
import sys

import rust_json_parser as rjp


USAGE = "usage: python -m rust_json_parser [-h | --help | --benchmark | INPUT]"


def read_input(input_data):
    """Read an existing UTF-8 file or return the supplied JSON text unchanged."""
    if os.path.isfile(input_data):
        with open(input_data, 'r', encoding='utf-8') as file:
            return file.read()
    return input_data


try:
    if len(sys.argv) > 2:
        print(USAGE, file=sys.stderr)
        print("Error: supply only one JSON input, file path, or option", file=sys.stderr)
        sys.exit(2)

    if len(sys.argv) > 1 and sys.argv[1] in ("-h", "--help"):
        print(USAGE)
        print("\nPretty-print JSON or benchmark parsers.")
        print("\nINPUT        JSON string or UTF-8 file path; omit to read piped input")
        print("-h, --help   Show this help message")
        print("--benchmark  Benchmark a fixed JSON sample")
        sys.exit(0)

    if len(sys.argv) > 1 and sys.argv[1] == "--benchmark":
        small = '{"key": "value"}'
        rust_time, python_json_time, simplejson_time = rjp.benchmark_performance(small)

        print(f"Rust:             {rust_time:.6f}s")
        for name, duration in [
            ("Python json (C)", python_json_time),
            ("simplejson", simplejson_time),
        ]:
            if rust_time < duration:
                comparison = f"Rust is {duration / rust_time:.2f}x faster"
            elif rust_time > duration:
                comparison = f"Rust is {rust_time / duration:.2f}x slower"
            else:
                comparison = "Rust is equally fast"
            print(f"{name + ':':<17}{duration:.6f}s  ({comparison})")
    else:
        if len(sys.argv) > 1:
            input_data = read_input(sys.argv[1])
        elif not sys.stdin.isatty():
            input_data = sys.stdin.read()
        else:
            print(USAGE, file=sys.stderr)
            sys.exit(1)
        parsed_json = rjp.parse_json(input_data)
        print(rjp.dumps(parsed_json, indent=2))
except IOError as e:
    print(f"Error: cannot read file - {e}", file=sys.stderr)
    sys.exit(1)
except ValueError as e:
    print(f"Error: Invalid JSON - {e}", file=sys.stderr)
    sys.exit(1)
except Exception as e:
    print(f"An unexpected error occurred: {e}", file=sys.stderr)
    sys.exit(1)
