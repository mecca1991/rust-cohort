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
            '"' => {
                chars.next();
                let mut collected = String::new();
                let mut nchar = chars.next().unwrap();
                while nchar != '"' {
                    collected.push(nchar);
                    nchar = chars.next().unwrap();
                }
                tokens.push(Token::String(collected));
            }
            ':' => {
                tokens.push(Token::Colon);
                chars.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            ch if ch.is_numeric() || (ch == '-') || (ch == '.') => {
                let mut num_string = String::new();
                chars.next();
                let mut nchar = ch;
                while nchar.is_numeric() || (nchar == '-') || (nchar == '.') {
                    if num_string.starts_with(".") {
                        num_string.clear();
                        break;
                    }
                    num_string.push(nchar);
                    let next = chars.peek();
                    if next.is_some() {
                        nchar = *next.unwrap();
                        if nchar.is_numeric() || nchar == '.' || nchar == '-' {
                            chars.next();
                        }
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
                let mut current_char = ch;
                chars.next();

                let valid_vals = vec!["true".to_string(), "false".to_string()];

                let mut bool_str = String::new();

                while !valid_vals.contains(&bool_str) {
                    bool_str.push(current_char);
                    let next_char = chars.peek();
                    if next_char.is_some() {
                        current_char = *next_char.unwrap();
                        if current_char.is_alphabetic() {
                            chars.next();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                if &bool_str == "null" {
                    tokens.push(Token::Null);
                } else if valid_vals.contains(&&bool_str) {
                    tokens.push(Token::Boolean(bool_str.parse().unwrap()));
                    bool_str.clear();
                }
            }
            _ => {
                println!("Token {ch} skipped...");
                chars.next();
            }
        }
    }
    return tokens;
}
