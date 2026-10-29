use rust_json_parser::JsonParser;
/// This is the main entry point for the Rust JSON parser application.
/// It demonstrates how to use the `JsonParser` to parse a JSON string and handle the result or any errors that may occur during parsing. The example input is a simple JSON string, and the output will display either the parsed result or an error message.
/// # Example
/// ```
/// let input = r#""The quick brown fox jumps over the lazy dog""#;
/// let result = JsonParser::new(input);
/// ```
fn main() {

    let input = r#""The quick brown fox jumps over the lazy dog""#;
    let result = JsonParser::new(input);
    match result {
        Ok(mut parser) => {
            let final_result = parser.parse();
            println!("The result is: \n{:?}", final_result)
        }
        Err(result) => {
            println!("An error occured! \n{:?}", result)
        }
    }
}
