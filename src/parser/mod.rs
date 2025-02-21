use crate::{
    ast::{Expr, ExprBinary},
    lex::lexer::{Token, TokenType},
};

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    position: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn peek(&self) -> &Token<'a> {
        // TODO: Check if option is better
        &self.tokens[self.position]
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenType::EOF
    }

    fn previous(&self) -> &Token<'a> {
        &self.tokens[self.position - 1]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.position += 1;
        }
        self.previous()
    }

    fn check(&self, kind: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().kind == kind
    }

    fn match_l(&mut self, kinds: Vec<TokenType>) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let expr = self.comparison();

        while self.match_l(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.comparison();
            expr = Expr::Binary(ExprBinary::new(Box::new(expr), operator, Box::new(right)));
        }

        expr
    }

    fn comparison(&mut self) -> Expr {
        todo!()
    }
}
