import sys
import os.path
import rust_json_parser as rjp

def handle_input(input_data):
    if os.path.isfile(input_data):
        result = rjp.parse_json_file(input_data)
    else:
        result = rjp.parse_json(input_data)
    return result


try:
    if len(sys.argv) > 1:
        input_arg = sys.argv[1]
        result = handle_input(input_arg)
        print(rjp.dumps(result, indent=4))
    elif len(sys.argv) == 1 and not sys.stdin.isatty():
        result = rjp.parse_json(sys.stdin.read())
        print(rjp.dumps(result, indent=4))
    else:
        print("Usage: script.py [FILE | JSON_STRING]", file=sys.stderr)
        sys.exit(1)
except IOError as e:
    print(f"Error: cannot read file - {e}", file=sys.stderr)
    sys.exit(1)
except ValueError as e:
    print(f"Error: Invalid JSON - {e}", file=sys.stderr)
    sys.exit(1)
except Exception as e:
    print(f"An unexpected error occurred: {e}", file=sys.stderr)
    sys.exit(1)