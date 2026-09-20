
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    String(String),
    Number,
    Boolean
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(&ch) = chars.peek() {
        match ch {
            '{' => {
                tokens.push(Token::LeftBrace);
                chars.next();
            },
            '}' => {
                tokens.push(Token::RightBrace);
                chars.next();

            },
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
            _ => {
                chars.next();
            }
        }
    }
    return tokens;
}
