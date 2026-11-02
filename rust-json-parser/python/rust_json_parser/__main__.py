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
        elif "--benchmark" in sys.argv:
            # Generate test data and run benchmarks
            small = '{"key": "value"}'
            rust_time, python_json_time, simplejson_time = rjp.benchmark_performance(small)
            # Calculate speedups
            json_speedup = python_json_time / rust_time
            simplejson_speedup = simplejson_time / rust_time

            print(f"Rust:             {rust_time:.6f}s")
            print(f"Python json (C):  {python_json_time:.6f}s  (Rust is {json_speedup:.2f}x faster)")
            print(f"simplejson:       {simplejson_time:.6f}s  (Rust is {simplejson_speedup:.2f}x faster)")
        else:
            result = rjp.parse_json(user_input)
    else:
        result = rjp.parse_json(sys.stdin.read())

    print(rjp.dumps(result, indent=2))
