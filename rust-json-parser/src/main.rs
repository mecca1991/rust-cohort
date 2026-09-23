use rust_json_parser::tokenizer::tokenize;

fn main() {
    let input = r#"{"age": 30, "children_names": ["Naia", "Bryan"]}"#;

    let tokens = tokenize(input);
    println!("Input JSON: {input}");
    println!("\nTokens:");
    println!("{:?}", tokens);
}
