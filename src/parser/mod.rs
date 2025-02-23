pub mod error;

use crate::{
    ast::{Expr, ExprBinary, ExprGrouping, ExprLiteral, ExprUnary, LiteralValue},
    lex::lexer::{Token, TokenType},
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

    fn peek(&self) -> &Token<'a> {
        &self.tokens[self.position]
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenType::EOF
    }

    fn previous(&self) -> Token<'a> {
        self.tokens[self.position - 1]
    }

    fn advance(&mut self) -> Token<'a> {
        if !self.is_at_end() {
            self.position += 1;
        }
        self.previous()
    }

    fn check(&self, kind: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        &self.peek().kind == kind
    }

    fn match_l(&mut self, kinds: &[TokenType]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    // fn match_l_token(&mut self, kinds: &[TokenType]) -> Option<Token<'a>> {
    //     for kind in kinds {
    //         if self.check(kind) {
    //             self.advance();
    //             return Some(self.previous());
    //         }
    //     }
    //     None
    // }

    fn consume(&mut self, kind: &TokenType, message: &str) {
        if self.check(kind) {
            self.advance();
            return;
        }

        //error
    }
}

pub struct Parser<'a> {
    tokenstream: TokenStream<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(tokenstream: TokenStream<'a>) -> Self {
        Self { tokenstream }
    }

    fn expression(&mut self) -> Expr<'a> {
        self.equality()
    }

    fn equality(&mut self) -> Expr<'a> {
        let mut expr = self.comparison();

        let operators = [TokenType::BangEqual, TokenType::EqualEqual];

        while self.tokenstream.match_l(&operators) {
            let operator = self.tokenstream.previous();
            let right = self.comparison();
            expr = Expr::Binary(ExprBinary::new(Box::new(expr), operator, Box::new(right)));
        }

        expr
    }

    fn comparison(&mut self) -> Expr<'a> {
        let mut expr = self.term();

        let operators = [
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ];

        while self.tokenstream.match_l(&operators) {
            let operator = self.tokenstream.previous();
            let right = self.term();
            expr = Expr::Binary(ExprBinary::new(Box::new(expr), operator, Box::new(right)));
        }

        expr
    }

    fn term(&mut self) -> Expr<'a> {
        let mut expr = self.factor();

        let operators = [TokenType::Minus, TokenType::Plus];

        while self.tokenstream.match_l(&operators) {
            let operator = self.tokenstream.previous();
            let right = self.factor();
            expr = Expr::Binary(ExprBinary::new(Box::new(expr), operator, Box::new(right)))
        }

        expr
    }

    fn factor(&mut self) -> Expr<'a> {
        let mut expr = self.unary();

        let operators = [TokenType::Slash, TokenType::Star];

        while self.tokenstream.match_l(&operators) {
            let operator = self.tokenstream.previous();
            let right = self.unary();
            expr = Expr::Binary(ExprBinary::new(Box::new(expr), operator, Box::new(right)))
        }

        expr
    }

    fn unary(&mut self) -> Expr<'a> {
        let operators = [TokenType::Bang, TokenType::Minus];

        if self.tokenstream.match_l(&operators) {
            let operator = self.tokenstream.previous();
            let right = self.unary();
            return Expr::Unary(ExprUnary::new(operator, Box::new(right)));
        }

        self.primary()
    }

    fn primary(&mut self) -> Expr<'a> {
        match self.tokenstream.peek().kind {
            TokenType::False => Expr::Literal(ExprLiteral::new(LiteralValue::Bool(false))),
            TokenType::True => Expr::Literal(ExprLiteral::new(LiteralValue::Bool(true))),
            TokenType::Nil => Expr::Literal(ExprLiteral::new(LiteralValue::Nil)),
            TokenType::Number(val) => Expr::Literal(ExprLiteral::new(LiteralValue::F64(val))),
            TokenType::String => Expr::Literal(ExprLiteral::new(LiteralValue::String(
                self.tokenstream.previous().lexeme.to_string(),
            ))),
            TokenType::LeftParen => {
                let expr = self.expression();
                self.tokenstream
                    .consume(&TokenType::RightParen, "Expected ')' after expression");
                return Expr::Grouping(ExprGrouping::new(Box::new(expr)));
            }
            _ => todo!(),
        }
    }
}
