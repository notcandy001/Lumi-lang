// ============================================================
//  Lumi Language — Lexer  (v0.2)
//  Converts raw .lu source text into a stream of Tokens.
// ============================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Create, Is, On, If, Else, While, Print, Say, Let, Set, Return,
    True, False, And, Or, Not,
    Equals, Greater, Less, Than,

    // Identifiers + Literals
    Identifier(String),
    StringLit(String),
    NumberLit(f64),

    // Arithmetic
    Plus, Minus, Star, Slash, Percent,

    // Comparison
    EqEq, NotEq, Lt, Gt, LtEq, GtEq,

    // Grouping
    LParen, RParen,

    // Structure
    Colon, Newline, Indent, Dedent, Eof,
}

#[derive(Debug, Clone)]
pub struct LexerError {
    pub line: usize,
    pub col:  usize,
    pub message: String,
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LexerError [line {}, col {}]: {}", self.line, self.col, self.message)
    }
}

pub fn lex(source: &str) -> Result<Vec<Token>, LexerError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut indent_stack: Vec<usize> = vec![0];

    for (line_no, raw_line) in source.lines().enumerate() {
        let line_number = line_no + 1;
        let trimmed_end = raw_line.trim_end();
        if trimmed_end.is_empty() || trimmed_end.trim_start().starts_with('#') {
            continue;
        }

        let indent = raw_line.len() - raw_line.trim_start().len();
        let current_indent = *indent_stack.last().unwrap();

        if indent > current_indent {
            indent_stack.push(indent);
            tokens.push(Token::Indent);
        } else {
            while indent < *indent_stack.last().unwrap() {
                indent_stack.pop();
                tokens.push(Token::Dedent);
            }
            if indent != *indent_stack.last().unwrap() {
                return Err(LexerError {
                    line: line_number, col: 1,
                    message: format!(
                        "Indentation mismatch: expected {} spaces, got {}",
                        indent_stack.last().unwrap(), indent
                    ),
                });
            }
        }

        let line_tokens = tokenize_line(trimmed_end.trim_start(), line_number)?;
        tokens.extend(line_tokens);
        tokens.push(Token::Newline);
    }

    while indent_stack.len() > 1 {
        indent_stack.pop();
        tokens.push(Token::Dedent);
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}

fn tokenize_line(line: &str, line_no: usize) -> Result<Vec<Token>, LexerError> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let col = i + 1;
        match chars[i] {
            ' ' | '\t' => { i += 1; }
            '#' => break,
            ':' => { tokens.push(Token::Colon);   i += 1; }
            '(' => { tokens.push(Token::LParen);  i += 1; }
            ')' => { tokens.push(Token::RParen);  i += 1; }
            '+' => { tokens.push(Token::Plus);    i += 1; }
            '*' => { tokens.push(Token::Star);    i += 1; }
            '/' => { tokens.push(Token::Slash);   i += 1; }
            '%' => { tokens.push(Token::Percent); i += 1; }
            '-' => {
                let prev_is_value = matches!(
                    tokens.last(),
                    Some(Token::NumberLit(_)) | Some(Token::StringLit(_))
                    | Some(Token::Identifier(_)) | Some(Token::RParen)
                    | Some(Token::True) | Some(Token::False)
                );
                if !prev_is_value && i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
                    i += 1;
                    let (num, new_i) = read_number(&chars, i, line_no, true)?;
                    tokens.push(num);
                    i = new_i;
                } else {
                    tokens.push(Token::Minus);
                    i += 1;
                }
            }
            '=' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::EqEq); i += 2;
                } else {
                    return Err(LexerError { line: line_no, col,
                        message: "Unexpected '='. Use 'is' for assignment or '==' for comparison.".into() });
                }
            }
            '!' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::NotEq); i += 2;
                } else {
                    return Err(LexerError { line: line_no, col,
                        message: "Unexpected '!'. Use 'not' for negation or '!=' for not-equal.".into() });
                }
            }
            '<' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::LtEq); i += 2;
                } else { tokens.push(Token::Lt); i += 1; }
            }
            '>' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::GtEq); i += 2;
                } else { tokens.push(Token::Gt); i += 1; }
            }
            '"' | '\'' => {
                let quote = chars[i];
                i += 1;
                let mut s = String::new();
                loop {
                    if i >= chars.len() {
                        return Err(LexerError { line: line_no, col,
                            message: "Unterminated string literal".into() });
                    }
                    match chars[i] {
                        c if c == quote => { i += 1; break; }
                        '\\' => {
                            i += 1;
                            if i >= chars.len() {
                                return Err(LexerError { line: line_no, col,
                                    message: "Unterminated escape sequence".into() });
                            }
                            match chars[i] {
                                'n'  => { s.push('\n'); i += 1; }
                                't'  => { s.push('\t'); i += 1; }
                                '"'  => { s.push('"');  i += 1; }
                                '\'' => { s.push('\''); i += 1; }
                                '\\' => { s.push('\\'); i += 1; }
                                c    => { s.push('\\'); s.push(c); i += 1; }
                            }
                        }
                        c => { s.push(c); i += 1; }
                    }
                }
                tokens.push(Token::StringLit(s));
            }
            c if c.is_ascii_digit() => {
                let (num, new_i) = read_number(&chars, i, line_no, false)?;
                tokens.push(num);
                i = new_i;
            }
            c if c.is_alphabetic() || c == '_' => {
                let mut word = String::new();
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    word.push(chars[i]);
                    i += 1;
                }
                tokens.push(keyword_or_ident(word));
            }
            c => {
                return Err(LexerError { line: line_no, col,
                    message: format!("Unexpected character: '{}'", c) });
            }
        }
    }
    Ok(tokens)
}

fn read_number(chars: &[char], mut i: usize, line_no: usize, negative: bool)
    -> Result<(Token, usize), LexerError>
{
    let mut num = if negative { "-".to_string() } else { String::new() };
    let mut has_dot = false;
    while i < chars.len() {
        match chars[i] {
            '.' if !has_dot && i + 1 < chars.len() && chars[i + 1].is_ascii_digit() => {
                has_dot = true; num.push('.'); i += 1;
            }
            c if c.is_ascii_digit() => { num.push(c); i += 1; }
            _ => break,
        }
    }
    match num.parse::<f64>() {
        Ok(n)  => Ok((Token::NumberLit(n), i)),
        Err(_) => Err(LexerError { line: line_no, col: i,
            message: format!("Invalid number: '{}'", num) }),
    }
}

fn keyword_or_ident(word: String) -> Token {
    match word.as_str() {
        "create"  => Token::Create,
        "is"      => Token::Is,
        "on"      => Token::On,
        "if"      => Token::If,
        "else"    => Token::Else,
        "while"   => Token::While,
        "print"   => Token::Print,
        "say"     => Token::Say,
        "let"     => Token::Let,
        "set"     => Token::Set,
        "return"  => Token::Return,
        "true"    => Token::True,
        "false"   => Token::False,
        "and"     => Token::And,
        "or"      => Token::Or,
        "not"     => Token::Not,
        "equals"  => Token::Equals,
        "greater" => Token::Greater,
        "less"    => Token::Less,
        "than"    => Token::Than,
        _         => Token::Identifier(word),
    }
}
