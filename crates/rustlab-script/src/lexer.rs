use crate::error::ScriptError;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Number(f64),
    Str(String),
    Ident(String),
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    DotStar,      // .*
    DotSlash,     // ./
    DotCaret,     // .^
    Colon,        // :
    Apostrophe,      // ' (conjugate transpose)
    DotApostrophe,   // .' (non-conjugate transpose)
    // Comparison operators
    EqEq,     // ==
    BangEq,   // !=
    Lt,       // <
    LtEq,     // <=
    Gt,       // >
    GtEq,     // >=
    // Logical operators
    AmpAmp,   // &&
    PipePipe, // ||
    Bang,     // !
    At,        // @
    // Delimiters
    Eq,       // =
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Semicolon,
    // Keywords
    Function,   // function
    End,        // end
    Return,     // return
    If,         // if
    Else,       // else
    For,        // for
    While,      // while
    ElseIf,     // elseif
    Switch,     // switch
    Case,       // case
    Otherwise,  // otherwise
    Run,        // run
    Dot,        // . (field access)
    // Structure
    Newline,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Spanned {
    pub token: Token,
    pub line:  usize,
}

pub fn tokenize(source: &str) -> Result<Vec<Spanned>, ScriptError> {
    let mut tokens: Vec<Spanned> = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let mut pos = 0;
    let mut line = 1usize;

    while pos < chars.len() {
        let ch = chars[pos];

        match ch {
            ' ' | '\t' | '\r' => {
                pos += 1;
            }
            '#' | '%' => {
                // Comment: skip until newline (don't consume the newline)
                while pos < chars.len() && chars[pos] != '\n' {
                    pos += 1;
                }
            }
            // Line continuation: ... skips rest of line and the newline
            '.' if pos + 2 < chars.len() && chars[pos + 1] == '.' && chars[pos + 2] == '.' => {
                pos += 3;
                // Skip rest of line (treated as comment)
                while pos < chars.len() && chars[pos] != '\n' {
                    pos += 1;
                }
                // Consume the newline but don't emit a Newline token
                if pos < chars.len() && chars[pos] == '\n' {
                    line += 1;
                    pos += 1;
                }
            }
            '\n' => {
                // Collapse consecutive newlines
                if tokens.last().map(|t| &t.token) != Some(&Token::Newline) {
                    tokens.push(Spanned { token: Token::Newline, line });
                }
                line += 1;
                pos += 1;
            }
            '+' => { tokens.push(Spanned { token: Token::Plus,       line }); pos += 1; }
            '-' => { tokens.push(Spanned { token: Token::Minus,     line }); pos += 1; }
            '*' => { tokens.push(Spanned { token: Token::Star,      line }); pos += 1; }
            '/' => { tokens.push(Spanned { token: Token::Slash,     line }); pos += 1; }
            '^' => { tokens.push(Spanned { token: Token::Caret,     line }); pos += 1; }
            ':' => { tokens.push(Spanned { token: Token::Colon,     line }); pos += 1; }
            '\'' => {
                // Context-dependent: transpose after ), ], Ident, Number;
                // otherwise start a single-quoted string literal.
                let is_transpose = matches!(
                    tokens.last().map(|t| &t.token),
                    Some(Token::RParen) | Some(Token::RBracket) |
                    Some(Token::Ident(_)) | Some(Token::Number(_)) |
                    Some(Token::Apostrophe) | Some(Token::DotApostrophe)
                );
                if is_transpose {
                    tokens.push(Spanned { token: Token::Apostrophe, line });
                    pos += 1;
                } else {
                    // Single-quoted string literal
                    pos += 1; // skip opening '
                    let start = pos;
                    while pos < chars.len() && chars[pos] != '\'' {
                        if chars[pos] == '\n' {
                            return Err(ScriptError::Lex {
                                line,
                                msg: "unterminated string literal".to_string(),
                            });
                        }
                        pos += 1;
                    }
                    if pos >= chars.len() {
                        return Err(ScriptError::Lex {
                            line,
                            msg: "unterminated string literal".to_string(),
                        });
                    }
                    let s: String = chars[start..pos].iter().collect();
                    tokens.push(Spanned { token: Token::Str(s), line });
                    pos += 1; // consume closing '
                }
            }
            '=' if pos + 1 < chars.len() && chars[pos + 1] == '=' => {
                tokens.push(Spanned { token: Token::EqEq,    line }); pos += 2;
            }
            '=' => { tokens.push(Spanned { token: Token::Eq,        line }); pos += 1; }
            '!' if pos + 1 < chars.len() && chars[pos + 1] == '=' => {
                tokens.push(Spanned { token: Token::BangEq,  line }); pos += 2;
            }
            '!' => { tokens.push(Spanned { token: Token::Bang,      line }); pos += 1; }
            '@' => { tokens.push(Spanned { token: Token::At,        line }); pos += 1; }
            '<' if pos + 1 < chars.len() && chars[pos + 1] == '=' => {
                tokens.push(Spanned { token: Token::LtEq,    line }); pos += 2;
            }
            '<' => { tokens.push(Spanned { token: Token::Lt,        line }); pos += 1; }
            '>' if pos + 1 < chars.len() && chars[pos + 1] == '=' => {
                tokens.push(Spanned { token: Token::GtEq,    line }); pos += 2;
            }
            '>' => { tokens.push(Spanned { token: Token::Gt,        line }); pos += 1; }
            '&' if pos + 1 < chars.len() && chars[pos + 1] == '&' => {
                tokens.push(Spanned { token: Token::AmpAmp,  line }); pos += 2;
            }
            '|' if pos + 1 < chars.len() && chars[pos + 1] == '|' => {
                tokens.push(Spanned { token: Token::PipePipe, line }); pos += 2;
            }
            '(' => { tokens.push(Spanned { token: Token::LParen,    line }); pos += 1; }
            ')' => { tokens.push(Spanned { token: Token::RParen,    line }); pos += 1; }
            '[' => { tokens.push(Spanned { token: Token::LBracket,  line }); pos += 1; }
            ']' => { tokens.push(Spanned { token: Token::RBracket,  line }); pos += 1; }
            ',' => { tokens.push(Spanned { token: Token::Comma,     line }); pos += 1; }
            ';' => { tokens.push(Spanned { token: Token::Semicolon, line }); pos += 1; }
            // Dot operators (.*  ./  .^  .') must be checked before the number branch
            '.' if pos + 1 < chars.len() && chars[pos + 1] == '*' => {
                tokens.push(Spanned { token: Token::DotStar,        line }); pos += 2;
            }
            '.' if pos + 1 < chars.len() && chars[pos + 1] == '/' => {
                tokens.push(Spanned { token: Token::DotSlash,       line }); pos += 2;
            }
            '.' if pos + 1 < chars.len() && chars[pos + 1] == '^' => {
                tokens.push(Spanned { token: Token::DotCaret,       line }); pos += 2;
            }
            '.' if pos + 1 < chars.len() && chars[pos + 1] == '\'' => {
                tokens.push(Spanned { token: Token::DotApostrophe,  line }); pos += 2;
            }
            // Field access: . followed by an identifier character
            '.' if pos + 1 < chars.len() && (chars[pos + 1].is_alphabetic() || chars[pos + 1] == '_') => {
                tokens.push(Spanned { token: Token::Dot, line }); pos += 1;
            }
            '"' => {
                // String literal
                pos += 1;
                let start = pos;
                while pos < chars.len() && chars[pos] != '"' {
                    if chars[pos] == '\n' {
                        return Err(ScriptError::Lex {
                            line,
                            msg: "unterminated string literal".to_string(),
                        });
                    }
                    pos += 1;
                }
                if pos >= chars.len() {
                    return Err(ScriptError::Lex {
                        line,
                        msg: "unterminated string literal".to_string(),
                    });
                }
                let s: String = chars[start..pos].iter().collect();
                tokens.push(Spanned { token: Token::Str(s), line });
                pos += 1; // consume closing "
            }
            c if c.is_ascii_digit() || c == '.' => {
                // Number
                let start = pos;
                while pos < chars.len() && (chars[pos].is_ascii_digit() || chars[pos] == '.') {
                    pos += 1;
                }
                // Optional exponent: e or E, optional sign
                if pos < chars.len() && (chars[pos] == 'e' || chars[pos] == 'E') {
                    pos += 1;
                    if pos < chars.len() && (chars[pos] == '+' || chars[pos] == '-') {
                        pos += 1;
                    }
                    while pos < chars.len() && chars[pos].is_ascii_digit() {
                        pos += 1;
                    }
                }
                let num_str: String = chars[start..pos].iter().collect();
                let val: f64 = num_str.parse().map_err(|_| ScriptError::Lex {
                    line,
                    msg: format!("invalid number: {}", num_str),
                })?;
                tokens.push(Spanned { token: Token::Number(val), line });
            }
            c if c.is_alphabetic() || c == '_' => {
                // Identifier or keyword
                let start = pos;
                while pos < chars.len() && (chars[pos].is_alphanumeric() || chars[pos] == '_') {
                    pos += 1;
                }
                let ident: String = chars[start..pos].iter().collect();
                let tok = match ident.as_str() {
                    "function" => Token::Function,
                    "end"      => Token::End,
                    "return"   => Token::Return,
                    "if"        => Token::If,
                    "elseif"   => Token::ElseIf,
                    "else"     => Token::Else,
                    "for"      => Token::For,
                    "while"    => Token::While,
                    "switch"   => Token::Switch,
                    "case"     => Token::Case,
                    "otherwise"=> Token::Otherwise,
                    "run"      => Token::Run,
                    _          => Token::Ident(ident),
                };
                tokens.push(Spanned { token: tok, line });
            }
            other => {
                return Err(ScriptError::Lex {
                    line,
                    msg: format!("unexpected character: {:?}", other),
                });
            }
        }
    }

    tokens.push(Spanned { token: Token::Eof, line });
    Ok(tokens)
}
