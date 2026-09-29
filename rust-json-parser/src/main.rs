use rust_json_parser::parse_json;

fn main() {
    let input = r#""The quick brown fox jumps over the lazy dog""#;

    let result = parse_json(input);
    match result {
        Ok(result) => {
            println!("The result is: \n{:?}", result)
        }
        Err(result) => {
            println!("An error occured! \n{:?}", result)
        }
    }
}
