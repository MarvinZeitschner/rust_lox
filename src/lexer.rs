use core::fmt;

#[derive(Debug)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Ident,
    String,
    Number(f64),

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    EOF,
}

#[derive(Debug)]
pub struct Token<'a> {
    kind: TokenType,
    lexeme: &'a str,
    line: u32,
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenType, lexeme: &'a str, line: u32) -> Self {
        Self { kind, lexeme, line }
    }
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lexeme = self.lexeme;
        match self.kind {
            TokenType::LeftParen => write!(f, "LeftParen {lexeme} null"),
            TokenType::RightParen => write!(f, "RightParen {lexeme} null"),
            TokenType::LeftBrace => write!(f, "LeftBrace {lexeme} null"),
            TokenType::RightBrace => write!(f, "RightBrace {lexeme} null"),
            TokenType::Comma => write!(f, "Comma {lexeme} null"),
            TokenType::Dot => write!(f, "Dot {lexeme} null"),
            TokenType::Minus => write!(f, "Minus {lexeme} null"),
            TokenType::Plus => write!(f, "Plus {lexeme} null"),
            TokenType::Semicolon => write!(f, "Semicolon {lexeme} null"),
            TokenType::Slash => write!(f, "Slash {lexeme} null"),
            TokenType::Star => write!(f, "Star {lexeme} null"),
            TokenType::Bang => write!(f, "Bang {lexeme} null"),
            TokenType::BangEqual => write!(f, "BangEqual {lexeme} null"),
            TokenType::Equal => write!(f, "Equal {lexeme} null"),
            TokenType::EqualEqual => write!(f, "EqualEqual {lexeme} null"),
            TokenType::Greater => write!(f, "Greater {lexeme} null"),
            TokenType::GreaterEqual => write!(f, "GreaterEqual {lexeme} null"),
            TokenType::Less => write!(f, "Less {lexeme} null"),
            TokenType::LessEqual => write!(f, "LessEqual {lexeme} null"),
            TokenType::Ident => write!(f, "Ident {lexeme} null"),
            TokenType::String => write!(f, "String {lexeme} null"),
            TokenType::Number(n) => write!(f, "Number {lexeme} {n}"),
            TokenType::And => write!(f, "And {lexeme} null"),
            TokenType::Class => write!(f, "Class {lexeme} null"),
            TokenType::Else => write!(f, "Else {lexeme} null"),
            TokenType::False => write!(f, "False {lexeme} null"),
            TokenType::Fun => write!(f, "Fun {lexeme} null"),
            TokenType::For => write!(f, "For {lexeme} null"),
            TokenType::If => write!(f, "If {lexeme} null"),
            TokenType::Nil => write!(f, "Nil {lexeme} null"),
            TokenType::Or => write!(f, "Or {lexeme} null"),
            TokenType::Print => write!(f, "Print {lexeme} null"),
            TokenType::Return => write!(f, "Return {lexeme} null"),
            TokenType::Super => write!(f, "Super {lexeme} null"),
            TokenType::This => write!(f, "This {lexeme} null"),
            TokenType::True => write!(f, "True {lexeme} null"),
            TokenType::Var => write!(f, "Var {lexeme} null"),
            TokenType::While => write!(f, "While {lexeme} null"),
            TokenType::EOF => write!(f, "EOF {lexeme} null"),
        }
    }
}

struct Scanner<'a> {
    source: &'a str,
    tokens: Vec<Token<'a>>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            tokens: vec![],
        }
    }
}
