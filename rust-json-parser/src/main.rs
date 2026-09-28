pub mod error;
mod value;
mod parser;
mod tokenizer;

use tokenizer::tokenize;

use crate::parser::parse_json;

fn main() {
    let input = r#"{"age": 30, "children_names": ["Naia", "Bryan"]}"#;

    let tokens = tokenize(input);
    let resp = parse_json(input);
    println!("Input JSON: {:?}", resp);
    println!("\nTokens:");
    println!("{:?}", tokens);
}
