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
            }
            ch if ('0'..='9').contains(&ch) | (ch == '-') | (ch == '.') => {
                let mut num_string = String::new();
                chars.next();
                let mut nchar = ch;
                while ('0'..='9').contains(&nchar) | (nchar == '-') | (nchar == '.') {
                    if num_string.starts_with(".") {
                        num_string.clear();
                        break;
                    }
                    num_string.push(nchar);
                    let next = chars.peek();
                    if next.is_some() {
                        nchar = *next.expect("msg");
                        chars.next();
                    } else {
                        break;
                    }
                }
                if !num_string.is_empty() {
                    tokens.push(Token::Number(num_string.parse::<f64>().unwrap()));
                }
            }
            _ => {
                chars.next();
            }
        }
    }
    return tokens;
}
