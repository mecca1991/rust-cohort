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
