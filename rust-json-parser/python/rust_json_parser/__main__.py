"""Pretty-print JSON or compare parsing times with Rust, json, and simplejson.

After installing the extension, pass JSON text or a file path::

    python -m rust_json_parser '{"name": "Alice"}'

The CLI reads existing files as UTF-8 and treats other inputs as JSON text.
Output uses two-space indentation. An explicit input takes precedence over
standard input. Without an input argument, the CLI reads piped input; in an
interactive terminal, it prints usage and exits with status 1.

To benchmark a UTF-8 JSON file::

    python -m rust_json_parser --benchmark FILE

Each parser parses the file's contents 1,000 times. Timings are totals in
seconds and exclude file reading. The output reports whether Rust is faster,
slower, or equally fast. Benchmark mode ignores standard input. Without a file
argument, it uses the fixed sample ``{"key": "value"}``.

Arguments are read directly from ``sys.argv``. Pretty-printing uses only the
first argument and ignores the rest. Benchmarking reads a file only when
exactly one argument follows ``--benchmark``; extra arguments make it use the
fixed sample instead. Use ``-h`` or ``--help`` to see the options.

Parsing and I/O errors go to standard error and cause exit status 1. Parse
failures and files that are not valid UTF-8 are reported as invalid JSON;
other read failures are reported as file errors.
"""

import os.path
import sys

import rust_json_parser as rjp


USAGE = "usage: python -m rust_json_parser [-h | --help | --benchmark [FILE] | INPUT]"


def read_input(input_data):
    """Read an existing UTF-8 file or return the supplied JSON text unchanged."""
    if os.path.isfile(input_data):
        with open(input_data, 'r', encoding='utf-8') as file:
            return file.read()
    return input_data


try:
    if len(sys.argv) > 1 and sys.argv[1] in ("-h", "--help"):
        print(USAGE)
        print("\nPretty-print JSON or compare parser timings.")
        print("\nINPUT               JSON text or UTF-8 file path; omit to read piped input")
        print("-h, --help          Show this help and exit")
        print("--benchmark [FILE]  Benchmark a UTF-8 JSON file; omit FILE for the fixed sample")
        sys.exit(0)

    if len(sys.argv) > 1 and sys.argv[1] == "--benchmark":
        if len(sys.argv) == 3:
            with open(sys.argv[2], "r", encoding="utf-8") as file:
                sample = file.read()
        else:
            sample = '{"key": "value"}'

        rust_time, python_json_time, simplejson_time = rjp.benchmark_performance(sample)

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
