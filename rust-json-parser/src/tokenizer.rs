#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(&ch) = chars.peek() {
        match ch {
            '{' => {
                tokens.push(Token::LeftBrace);
                chars.next();
            }
            '}' => {
                tokens.push(Token::RightBrace);
                chars.next();
            }
            '[' => {
                tokens.push(Token::LeftBracket);
                chars.next();
            }
            ']' => {
                tokens.push(Token::RightBracket);
                chars.next();
            }
            ':' => {
                tokens.push(Token::Colon);
                chars.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            '"' => {
                chars.next();
                let mut collected = String::new();
                for nchar in chars.by_ref() {
                    if nchar == '"' {
                        break;
                    }
                    collected.push(nchar);
                }
                tokens.push(Token::String(collected));
            }
            ch if ch.is_numeric() || (ch == '-') || (ch == '.') => {
                let mut num_string: String = String::new();
                while let Some(value) = chars.peek() {
                    if value.is_numeric() || ['-', '.'].contains(value) {
                        num_string.push(*value);
                        if num_string.starts_with(".") {
                            num_string.clear();
                        }
                        chars.next();
                    } else {
                        break;
                    }
                }
                if !num_string.is_empty() {
                    tokens.push(Token::Number(
                        num_string.parse::<f64>().expect("Invalid Number!"),
                    ));
                }
            }
            ch if ch == 't' || ch == 'f' || ch == 'n' => {
                let mut match_str = String::new();
                let boolean_values = ["true".to_string(), "false".to_string()];
                while let Some(nchar) = chars.peek() {
                    if nchar.is_alphabetic() && !boolean_values.contains(&match_str) {
                        match_str.push(*nchar);
                    }
                    if &match_str == "null" {
                        tokens.push(Token::Null);
                        match_str.clear();
                    }
                    if boolean_values.contains(&match_str) {
                        tokens.push(Token::Boolean(
                            match_str.parse().expect("Invalid Boolean value"),
                        ));
                        match_str.clear();
                    }
                    chars.next();
                }
            }
            ' ' => {
                chars.next();
            }
            _ => {
                println!("Token {ch} skipped...");
            }
        }
    }
    tokens
}
