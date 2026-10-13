use rust_json_parser::JsonParser;

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
