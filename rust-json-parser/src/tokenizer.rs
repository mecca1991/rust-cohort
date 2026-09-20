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
            '"' => {
                chars.next();
                let mut collected = String::new();
                let mut nchar = chars.next().expect("Val");
                while nchar != '"' {
                    collected.push(nchar);
                    nchar = chars.next().expect("u");
                }
                tokens.push(Token::String(collected));
            },
            ':' => {
                tokens.push(Token::Colon);
                chars.next();
            },
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            },
            ch if ch.is_numeric() | (ch == '-') | (ch == '.') => {
                let mut num_string = String::new();
                chars.next();
                let mut nchar = ch;
                while &nchar.is_numeric() | (nchar == '-') | (nchar == '.') {
                    if num_string.starts_with(".") {
                        num_string.clear();
                        break;
                    }
                    num_string.push(nchar);
                    let next = chars.peek();
                    if next.is_some() {
                        nchar = *next.expect("msg");
                        if nchar.is_numeric() || nchar == '.' || nchar == '-' {
                            chars.next();
                        } 
                    } else {
                        break;
                    }
                }
                if !num_string.is_empty() {
                    tokens.push(Token::Number(num_string.parse::<f64>().unwrap()));
                }
            }
            ch if ch.is_alphabetic() => {
                let mut current_char = ch;
                chars.next();
                let true_str = "true".to_string();
                let false_str = "false".to_string();
                let null_str = "null".to_string();

                let valid_vals = vec![&true_str, &false_str];

                let mut bool_str = String::new();

                while !valid_vals.contains(&&bool_str) {
                    bool_str.push(current_char);
                    let next_char = chars.peek();
                    if next_char.is_some() {
                        current_char = *next_char.expect("msg");
                        if current_char.is_alphabetic() {
                            chars.next();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                if &bool_str == &null_str {
                    tokens.push(Token::Null);
                } else if valid_vals.contains(&&bool_str) {
                    tokens.push(Token::Boolean(bool_str.parse().unwrap()));
                    bool_str.clear();
                }
            }
            _ => {
                chars.next();
            }
        }
    }
    return tokens;
}
