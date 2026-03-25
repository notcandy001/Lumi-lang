// ============================================================
//  Lumi Language — Lexer (Tokenizer)
//  Converts raw .lu source text into a stream of Tokens
// ============================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // ── Keywords ──────────────────────────────────────────
    Create,   // create
    Is,       // is
    On,       // on
    If,       // if
    Else,     // else
    While,    // while  (future)
    Print,    // print
    Let,      // let
    Set,      // set  (reassignment)
    True,     // true
    False,    // false
    And,      // and
    Or,       // or
    Not,      // not

    // ── Component / Event names (context-sensitive) ───────
    Identifier(String),

    // ── Literals ──────────────────────────────────────────
    StringLit(String),
    NumberLit(f64),

    // ── Structure ─────────────────────────────────────────
    Colon,      // :
    Newline,    // \n
    Indent,     // synthetic — increase
    Dedent,     // synthetic — decrease
    Eof,
}

#[derive(Debug, Clone)]
pub struct LexerError {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LexerError at line {}: {}", self.line, self.message)
    }
}

/// Lex the full source into a flat token list (with synthetic INDENT/DEDENT).
pub fn lex(source: &str) -> Result<Vec<Token>, LexerError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut indent_stack: Vec<usize> = vec![0];

    for (line_no, raw_line) in source.lines().enumerate() {
        let line_number = line_no + 1;

        // Skip blank / comment lines
        let trimmed = raw_line.trim_end();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Count leading spaces for indentation
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
                    line: line_number,
                    message: format!(
                        "Indentation mismatch: expected {}, got {}",
                        indent_stack.last().unwrap(),
                        indent
                    ),
                });
            }
        }

        // Tokenize the rest of the line
        let line_tokens = tokenize_line(trimmed.trim_start(), line_number)?;
        tokens.extend(line_tokens);
        tokens.push(Token::Newline);
    }

    // Close any remaining indents
    while indent_stack.len() > 1 {
        indent_stack.pop();
        tokens.push(Token::Dedent);
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}

fn tokenize_line(line: &str, line_no: usize) -> Result<Vec<Token>, LexerError> {
    let mut tokens = Vec::new();
    let mut chars = line.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            // Whitespace within a line
            ' ' | '\t' => { chars.next(); }

            // Comments
            '#' => break,

            // Colon
            ':' => { chars.next(); tokens.push(Token::Colon); }

            // String literals
            '"' => {
                chars.next(); // opening quote
                let mut s = String::new();
                loop {
                    match chars.next() {
                        Some('"') => break,
                        Some('\\') => {
                            match chars.next() {
                                Some('n') => s.push('\n'),
                                Some('t') => s.push('\t'),
                                Some('"') => s.push('"'),
                                Some('\\') => s.push('\\'),
                                Some(c) => s.push(c),
                                None => return Err(LexerError {
                                    line: line_no,
                                    message: "Unterminated string escape".into(),
                                }),
                            }
                        }
                        Some(c) => s.push(c),
                        None => return Err(LexerError {
                            line: line_no,
                            message: "Unterminated string literal".into(),
                        }),
                    }
                }
                tokens.push(Token::StringLit(s));
            }

            // Number literals
            '0'..='9' | '-' => {
                let mut num = String::new();
                if ch == '-' { num.push(ch); chars.next(); }
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() || d == '.' { num.push(d); chars.next(); }
                    else { break; }
                }
                match num.parse::<f64>() {
                    Ok(n) => tokens.push(Token::NumberLit(n)),
                    Err(_) => return Err(LexerError {
                        line: line_no,
                        message: format!("Invalid number: {}", num),
                    }),
                }
            }

            // Identifiers and keywords
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut word = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' { word.push(c); chars.next(); }
                    else { break; }
                }
                tokens.push(keyword_or_ident(word));
            }

            c => {
                return Err(LexerError {
                    line: line_no,
                    message: format!("Unexpected character: '{}'", c),
                });
            }
        }
    }

    Ok(tokens)
}

fn keyword_or_ident(word: String) -> Token {
    match word.as_str() {
        "create" => Token::Create,
        "is"     => Token::Is,
        "on"     => Token::On,
        "if"     => Token::If,
        "else"   => Token::Else,
        "while"  => Token::While,
        "print"  => Token::Print,
        "let"    => Token::Let,
        "set"    => Token::Set,
        "true"   => Token::True,
        "false"  => Token::False,
        "and"    => Token::And,
        "or"     => Token::Or,
        "not"    => Token::Not,
        _        => Token::Identifier(word),
    }
}
