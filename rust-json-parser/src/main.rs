//! Demonstrates parsing a fixed JSON string with the Rust library.
//!
//! Run `cargo run --no-default-features` from the crate directory to print the
//! parse result. This demo does not read arguments or standard input. The Python
//! package provides that interface through `python -m rust_json_parser`.

use rust_json_parser::JsonParser;

/// Parses the sample string and prints the result or a tokenization error.
fn main() {
    let input = r#""The quick brown fox jumps over the lazy dog""#;
    let result = JsonParser::new(input);
    match result {
        Ok(mut parser) => {
            let final_result = parser.parse();
            println!("The result is: \n{:?}", final_result)
        }
        Err(result) => {
            println!("An error occurred! \n{:?}", result)
        }
    }
}
