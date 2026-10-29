import sys
import os.path
import rust_json_parser as rjp

def handle_input(input_data):
    if os.path.isfile(input_data):
        result = rjp.parse_json_file(input_data)
    elif not sys.stdin.isatty():
        result = rjp.parse_json(sys.stdin.read())
    else:
        print("Usage: script.py [FILE | JSON_STRING]", file=sys.stderr)
        sys.exit(1)
    return result

try:
    input_arg = sys.argv[1] if len(sys.argv) > 1 else None
    input_data = input_arg if input_arg is not None else sys.stdin.read()
    result = handle_input(input_data)
    print(rjp.dumps(result, indent=4))
except OSError as e:
    print(f"Error: cannot read '{input_arg}' - {e}", file=sys.stderr)
    sys.exit(1)
except ValueError as e:
    print(f"Error: Invalid JSON - {e}", file=sys.stderr)
    sys.exit(1)
except Exception as e:
    print(f"An unexpected error occurred: {e}", file=sys.stderr)
    sys.exit(1)
