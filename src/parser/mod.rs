pub mod error;

use error::{ParserError, TokenStreamError};

use crate::{
    ast::{Expr, ExprBinary, ExprGrouping, ExprLiteral, ExprUnary, LiteralValue},
    lex::{Token, TokenType},
};

pub struct TokenStream<'a> {
    tokens: Vec<Token<'a>>,
    position: usize,
}

impl<'a> TokenStream<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn peek(&self) -> Result<&Token<'a>, TokenStreamError> {
        if self.position >= self.tokens.len() {
            return Err(TokenStreamError::OutOfBounds);
        }
        Ok(&self.tokens[self.position])
    }

    fn is_at_end(&self) -> bool {
        self.tokens[self.position].kind == TokenType::EOF
    }

    fn previous(&self) -> Result<Token<'a>, TokenStreamError> {
        if self.position == 0 {
            return Err(TokenStreamError::OutOfBounds);
        }
        Ok(self.tokens[self.position - 1])
    }

    fn advance(&mut self) -> Result<Token<'a>, TokenStreamError> {
        if self.position < self.tokens.len() - 1 {
            self.position += 1;
        }
        self.previous()
    }

    fn check(&self, kind: &TokenType) -> Result<bool, TokenStreamError> {
        if self.is_at_end() {
            return Ok(false);
        }

        Ok(&self.peek()?.kind == kind)
    }

    fn match_l(&mut self, kinds: &[TokenType]) -> Result<bool, TokenStreamError> {
        for kind in kinds {
            if self.check(kind)? {
                self.advance()?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn consume(&mut self, kind: &TokenType) -> Result<(), ParserError<'a>> {
        if self.check(kind)? {
            self.advance()?;
            return Ok(());
        }
        Err(ParserError::UnmatchedParanthesis {
            token: self.previous()?,
        })
    }
}

pub struct Parser<'a> {
    tokenstream: TokenStream<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(tokenstream: TokenStream<'a>) -> Self {
        Self { tokenstream }
    }

    pub fn parse(&mut self) -> Result<Expr<'a>, ParserError<'a>> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr<'a>, ParserError<'a>> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr<'a>, ParserError<'a>> {
        let mut expr = self.comparison()?;

        let operators = [TokenType::BangEqual, TokenType::EqualEqual];

        while self.tokenstream.match_l(&operators)? {
            let operator = self.tokenstream.previous()?;
            let right = self.comparison()?;
            expr = Expr::Binary(ExprBinary::new(Box::new(expr), operator, Box::new(right)));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr<'a>, ParserError<'a>> {
        let mut expr = self.term()?;

        let operators = [
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ];

        while self.tokenstream.match_l(&operators)? {
            let operator = self.tokenstream.previous()?;
            let right = self.term()?;
            expr = Expr::Binary(ExprBinary::new(Box::new(expr), operator, Box::new(right)));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr<'a>, ParserError<'a>> {
        let mut expr = self.factor()?;

        let operators = [TokenType::Minus, TokenType::Plus];

        while self.tokenstream.match_l(&operators)? {
            let operator = self.tokenstream.previous()?;
            let right = self.factor()?;
            expr = Expr::Binary(ExprBinary::new(Box::new(expr), operator, Box::new(right)))
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr<'a>, ParserError<'a>> {
        let mut expr = self.unary()?;

        let operators = [TokenType::Slash, TokenType::Star];

        while self.tokenstream.match_l(&operators)? {
            let operator = self.tokenstream.previous()?;
            let right = self.unary()?;
            expr = Expr::Binary(ExprBinary::new(Box::new(expr), operator, Box::new(right)))
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr<'a>, ParserError<'a>> {
        let operators = [TokenType::Bang, TokenType::Minus];

        if self.tokenstream.match_l(&operators)? {
            let operator = self.tokenstream.previous()?;
            let right = self.unary()?;
            return Ok(Expr::Unary(ExprUnary::new(operator, Box::new(right))));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr<'a>, ParserError<'a>> {
        let token = self.tokenstream.advance()?;
        match token.kind {
            TokenType::False => Ok(Expr::Literal(ExprLiteral::new(LiteralValue::Bool(false)))),
            TokenType::True => Ok(Expr::Literal(ExprLiteral::new(LiteralValue::Bool(true)))),
            TokenType::Nil => Ok(Expr::Literal(ExprLiteral::new(LiteralValue::Nil))),
            TokenType::Number(val) => Ok(Expr::Literal(ExprLiteral::new(LiteralValue::F64(val)))),
            TokenType::String => Ok(Expr::Literal(ExprLiteral::new(LiteralValue::String(
                self.tokenstream.previous()?.lexeme.to_string(),
            )))),
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.tokenstream.consume(&TokenType::RightParen)?;
                return Ok(Expr::Grouping(ExprGrouping::new(Box::new(expr))));
            }
            _ => Err(ParserError::UnexpectedToken { token }),
        }
    }

    fn synchronize(&mut self) -> Result<(), ParserError> {
        self.tokenstream.advance()?;

        while !self.tokenstream.is_at_end() {
            if self.tokenstream.previous()?.kind == TokenType::Semicolon {
                return Ok(());
            }

            match self.tokenstream.peek()?.kind {
                TokenType::Class => (),
                TokenType::Fun => (),
                TokenType::Var => (),
                TokenType::For => (),
                TokenType::If => (),
                TokenType::While => (),
                TokenType::Print => (),
                TokenType::Return => (),
                _ => {
                    self.tokenstream.advance()?;
                    return Ok(());
                }
            };
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::lex::{Scanner, Span};

    use super::*;
    fn setup(input: &str) -> Parser {
        let mut lexer = Scanner::new(input);
        Parser::new(TokenStream::new(lexer.scan_tokens().unwrap()))
    }

    #[test]
    fn ts_error() {
        let input = "2 + 3";
        let mut lexer = Scanner::new(input);
        let ts = TokenStream::new(lexer.scan_tokens().unwrap());
        let res = ts.previous();

        assert_eq!(Err(TokenStreamError::OutOfBounds), res);
    }

    #[test]
    fn recursive_descent() {
        let input = "2 + 3";
        let mut parser = setup(input);

        if let Ok(expr) = parser.parse() {
            let left = Expr::Literal(ExprLiteral::new(LiteralValue::F64(2.0)));

            let span = Span { begin: 2, end: 3 };
            let operator = Token::new(TokenType::Plus, "+", 1, span);

            let right = Expr::Literal(ExprLiteral::new(LiteralValue::F64(3.0)));

            assert_eq!(
                Expr::Binary(ExprBinary::new(Box::new(left), operator, Box::new(right))),
                expr
            );
        }
    }

    #[test]
    fn rd_error() {
        let input = "(1 + 1";
        let mut parser = setup(input);

        if let Err(expr) = parser.parse() {
            let span = Span { begin: 5, end: 6 };
            let right = Token::new(TokenType::Number(1.0), "1", 1, span);

            assert_eq!(expr, ParserError::UnmatchedParanthesis { token: right });
        }
    }
}
