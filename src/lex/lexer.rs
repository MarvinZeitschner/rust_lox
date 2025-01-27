use core::fmt;

#[derive(Debug, PartialEq, PartialOrd)]
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

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Span {
    begin: u32,
    end: u32,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Token<'a> {
    kind: TokenType,
    lexeme: &'a str,
    line: u32,
    span: Span,
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenType, lexeme: &'a str, line: u32, span: Span) -> Self {
        Self {
            kind,
            lexeme,
            line,
            span,
        }
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
            TokenType::String => write!(f, "String {lexeme} {lexeme}"),
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
    position: usize,
    start: usize,
    line: u32,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            position: 0,
            start: 0,
            line: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.source[self.position..].chars().next()
    }
    fn peek_nth(&self, nth: usize) -> Option<char> {
        self.source[self.position..].chars().nth(nth)
    }

    fn read_char(&mut self) -> Option<char> {
        if self.position < self.source.len() {
            let c = self.peek()?;
            self.position += c.len_utf8();
            Some(c)
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.read_char();
            } else {
                break;
            }
        }
    }

    fn make_token(&self, kind: TokenType) -> Token<'a> {
        let lexeme = &self.source[self.start..self.position];
        Token::new(
            kind,
            lexeme,
            self.line,
            Span {
                begin: self.start as u32,
                end: self.position as u32,
            },
        )
    }

    fn match_next(&mut self, expected: char) -> bool {
        if let Some(c) = self.peek() {
            if c == expected {
                self.read_char();
                return true;
            }
        }
        false
    }

    fn number(&mut self) -> Token<'a> {
        while let Some(c) = self.peek() {
            if !c.is_ascii_digit() {
                break;
            }
            self.read_char();
        }

        if let Some('.') = self.peek() {
            if let Some(next) = self.peek_nth(1) {
                if next.is_ascii_digit() {
                    self.read_char();

                    while let Some(c) = self.peek() {
                        if !c.is_ascii_digit() {
                            break;
                        }
                        self.read_char();
                    }
                }
            }
        }

        let value: f64 = self.source[self.start..self.position]
            .parse()
            .unwrap_or(0.0);
        self.make_token(TokenType::Number(value))
    }

    fn string(&mut self) -> Option<Token<'a>> {
        while let Some(c) = self.peek() {
            if c == '"' {
                break;
            }
            if c == '\n' {
                self.line += 1;
            }
            self.read_char();
        }

        self.peek()?;

        self.read_char();

        Some(self.make_token(TokenType::String))
    }
    fn identifier(&mut self) -> Token<'a> {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                break;
            }
            self.read_char();
        }

        let lexeme = &self.source[self.start..self.position];
        let kind = match lexeme {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Ident,
        };

        self.make_token(kind)
    }

    pub fn next(&mut self) -> Option<Token<'a>> {
        self.skip_whitespace();
        self.start = self.position;

        let c = self.read_char()?;
        let token = match c {
            '(' => self.make_token(TokenType::LeftParen),
            ')' => self.make_token(TokenType::RightParen),
            '{' => self.make_token(TokenType::LeftBrace),
            '}' => self.make_token(TokenType::RightBrace),
            ',' => self.make_token(TokenType::Comma),
            '.' => self.make_token(TokenType::Dot),
            '-' => self.make_token(TokenType::Minus),
            '+' => self.make_token(TokenType::Plus),
            '*' => self.make_token(TokenType::Star),
            ';' => self.make_token(TokenType::Semicolon),
            '!' => {
                let token = if self.match_next('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.make_token(token)
            }
            '=' => {
                let token = if self.match_next('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.make_token(token)
            }
            '<' => {
                let token = if self.match_next('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.make_token(token)
            }
            '>' => {
                let token = if self.match_next('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.make_token(token)
            }
            '/' => self.make_token(TokenType::Slash),
            // TODO: Maybe a result of token would be better to catch non terminated strings and
            // other composite types
            '"' => self.string()?,
            c if c.is_ascii_digit() => self.number(),
            c if c.is_alphabetic() => self.identifier(),
            _ => self.make_token(TokenType::EOF),
        };

        Some(token)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn peek() {
        let input = "(";
        let scanner = Scanner::new(input);
        assert_eq!(scanner.peek().unwrap(), '(');
    }

    #[test]
    fn whitespace() {
        let input = "  (";
        let mut scanner = Scanner::new(input);
        scanner.skip_whitespace();
        assert_eq!(scanner.peek().unwrap(), '(');
    }

    #[test]
    fn read_char() {
        let input = "(";
        let mut scanner = Scanner::new(input);
        assert_eq!(scanner.read_char().unwrap(), '(');
    }

    #[test]
    fn paren() {
        let input = "(";
        let mut scanner = Scanner::new(input);
        let token = Token::new(TokenType::LeftParen, "(", 1, Span { begin: 0, end: 1 });
        assert_eq!(token, scanner.next().unwrap());
    }

    #[test]
    fn composite() {
        let input = "!= ! !=";
        let mut scanner = Scanner::new(input);
        let mut span = Span { begin: 0, end: 2 };
        let mut token = Token::new(TokenType::BangEqual, "!=", 1, span);
        assert_eq!(token, scanner.next().unwrap());
        span = Span { begin: 3, end: 4 };
        token = Token::new(TokenType::Bang, "!", 1, span);
        assert_eq!(token, scanner.next().unwrap());
        span = Span { begin: 5, end: 7 };
        token = Token::new(TokenType::BangEqual, "!=", 1, span);
        assert_eq!(token, scanner.next().unwrap());
    }

    #[test]
    fn number() {
        let input = "1234.123 123";
        let mut scanner = Scanner::new(input);
        let mut span = Span { begin: 0, end: 8 };
        let mut token = Token::new(TokenType::Number(1234.123), "1234.123", 1, span);
        assert_eq!(token, scanner.next().unwrap());
        span = Span { begin: 9, end: 12 };
        token = Token::new(TokenType::Number(123.0), "123", 1, span);
        assert_eq!(token, scanner.next().unwrap());
    }

    #[test]
    fn string() {
        let input = "\"test\" \"test";
        let mut scanner = Scanner::new(input);
        let span = Span { begin: 0, end: 6 };
        let token = Token::new(TokenType::String, "\"test\"", 1, span);
        assert_eq!(token, scanner.next().unwrap());
        assert_eq!(None, scanner.next());
    }

    #[test]
    fn ident() {
        let input = "test t123 class";
        let mut scanner = Scanner::new(input);
        let mut span = Span { begin: 0, end: 4 };
        let mut token = Token::new(TokenType::Ident, "test", 1, span);
        assert_eq!(token, scanner.next().unwrap());
        span = Span { begin: 5, end: 9 };
        token = Token::new(TokenType::Ident, "t123", 1, span);
        assert_eq!(token, scanner.next().unwrap());
        span = Span { begin: 10, end: 15 };
        token = Token::new(TokenType::Class, "class", 1, span);
        assert_eq!(token, scanner.next().unwrap());
    }
}
