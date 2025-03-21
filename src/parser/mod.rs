pub mod error;

use error::{ParserError, ParserErrorContext, TokenStreamError};

use crate::{
    ast::{
        Expr, ExprAssign, ExprBinary, ExprCall, ExprGet, ExprGrouping, ExprLiteral, ExprLogical,
        ExprSet, ExprThis, ExprUnary, ExprVariable, LiteralValue, Stmt, StmtBlock, StmtClass,
        StmtExpression, StmtFunction, StmtIf, StmtPrint, StmtReturn, StmtVar, StmtWhile,
    },
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

    fn consume(
        &mut self,
        kind: &TokenType,
        error_context: ParserErrorContext,
    ) -> Result<Token<'a>, ParserError<'a>> {
        if self.check(kind)? {
            let token = self.advance()?;
            return Ok(token);
        }
        let err = error_context.to_error(self.previous()?);
        Err(err)
    }
}

pub struct Parser<'a> {
    tokenstream: TokenStream<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(tokenstream: TokenStream<'a>) -> Self {
        Self { tokenstream }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt<'a>>, ParserError<'a>> {
        let mut statements = vec![];
        while !self.tokenstream.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn try_with_sync<T, F>(&mut self, f: F) -> Result<T, ParserError<'a>>
    where
        F: FnOnce(&mut Self) -> Result<T, ParserError<'a>>,
    {
        match f(self) {
            Ok(res) => Ok(res),
            Err(e) => {
                self.synchronize()?;
                Err(e)
            }
        }
    }

    fn declaration(&mut self) -> Result<Stmt<'a>, ParserError<'a>> {
        if self.tokenstream.match_l(&[TokenType::Var])? {
            return self.try_with_sync(|s| s.var_declaration());
        }

        if self.tokenstream.match_l(&[TokenType::Class])? {
            return self.try_with_sync(|s| s.class_declaration());
        }

        if self.tokenstream.match_l(&[TokenType::Fun])? {
            return self.try_with_sync(|s| s.function(ParserErrorContext::ExpectedFunctionName));
        }

        self.statement()
    }

    fn class_declaration(&mut self) -> Result<Stmt<'a>, ParserError<'a>> {
        let name = self
            .tokenstream
            .consume(&TokenType::Ident, ParserErrorContext::ExpectedClassName)?;
        self.tokenstream.consume(
            &TokenType::LeftBrace,
            ParserErrorContext::ExpectedLeftBraceBeforeClassBody,
        )?;

        let mut methods = vec![];

        while !self.tokenstream.check(&TokenType::RightBrace)? && !self.tokenstream.is_at_end() {
            let Stmt::Function(fun) = self.function(ParserErrorContext::ExpectedMethod)? else {
                // TODO: Handle errror
                panic!("not a function");
            };

            methods.push(fun);
        }
        self.tokenstream.consume(
            &TokenType::RightBrace,
            ParserErrorContext::ExpectedRightBraceAfterClassBody,
        )?;

        Ok(Stmt::Class(StmtClass::new(name, methods)))
    }

    fn var_declaration(&mut self) -> Result<Stmt<'a>, ParserError<'a>> {
        let name = self.tokenstream.consume(
            &TokenType::Ident,
            ParserErrorContext::InvalidAssignmentTarget,
        )?;
        let mut initializer = None;
        if self.tokenstream.match_l(&[TokenType::Equal])? {
            initializer = Some(self.expression()?);
        }
        self.tokenstream
            .consume(&TokenType::Semicolon, ParserErrorContext::ExpectedSemicolon)?;
        Ok(Stmt::Var(StmtVar::new(name, initializer)))
    }

    fn statement(&mut self) -> Result<Stmt<'a>, ParserError<'a>> {
        if self.tokenstream.match_l(&[TokenType::If])? {
            return self.if_statement();
        }
        if self.tokenstream.match_l(&[TokenType::Print])? {
            return self.print_statement();
        }
        if self.tokenstream.match_l(&[TokenType::Return])? {
            return self.return_statement();
        }
        if self.tokenstream.match_l(&[TokenType::LeftBrace])? {
            return Ok(Stmt::Block(StmtBlock::new(self.block()?)));
        }
        if self.tokenstream.match_l(&[TokenType::While])? {
            return self.while_statement();
        }
        if self.tokenstream.match_l(&[TokenType::For])? {
            return self.for_statement();
        }

        self.expression_statement()
    }

    fn if_statement(&mut self) -> Result<Stmt<'a>, ParserError<'a>> {
        self.tokenstream.consume(
            &TokenType::LeftParen,
            ParserErrorContext::ExpectedLeftParenAfterIf,
        )?;
        let condition = self.expression()?;
        self.tokenstream.consume(
            &TokenType::RightParen,
            ParserErrorContext::ExpectedRightParenAfterCondition,
        )?;

        let then_branch = Box::new(self.statement()?);
        let mut else_branch = None;
        if self.tokenstream.match_l(&[TokenType::Else])? {
            else_branch = Some(Box::new(self.statement()?));
        }

        Ok(Stmt::If(StmtIf::new(condition, then_branch, else_branch)))
    }

    fn block(&mut self) -> Result<Vec<Stmt<'a>>, ParserError<'a>> {
        let mut statements = vec![];

        while !self.tokenstream.check(&TokenType::RightBrace)? && !self.tokenstream.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.tokenstream.consume(
            &TokenType::RightBrace,
            ParserErrorContext::ExpectedRightBrace,
        )?;

        Ok(statements)
    }

    fn print_statement(&mut self) -> Result<Stmt<'a>, ParserError<'a>> {
        let value = self.expression()?;
        self.tokenstream
            .consume(&TokenType::Semicolon, ParserErrorContext::ExpectedSemicolon)?;
        Ok(Stmt::Print(StmtPrint::new(value)))
    }

    fn return_statement(&mut self) -> Result<Stmt<'a>, ParserError<'a>> {
        let keyword = self.tokenstream.previous()?;
        let mut value = None;

        if !self.tokenstream.check(&TokenType::Semicolon)? {
            value = Some(self.expression()?);
        }

        self.tokenstream.consume(
            &TokenType::Semicolon,
            ParserErrorContext::ExpectedSemicolonAfterReturnValue,
        )?;

        Ok(Stmt::Return(StmtReturn::new(keyword, value)))
    }

    fn while_statement(&mut self) -> Result<Stmt<'a>, ParserError<'a>> {
        self.tokenstream.consume(
            &TokenType::LeftParen,
            ParserErrorContext::ExpectedLeftParenAfterWhile,
        )?;
        let condition = self.expression()?;
        self.tokenstream.consume(
            &TokenType::RightParen,
            ParserErrorContext::ExpectedRightParenAfterCondition,
        )?;

        let body = self.statement()?;

        Ok(Stmt::While(StmtWhile::new(condition, Box::new(body))))
    }

    fn for_statement(&mut self) -> Result<Stmt<'a>, ParserError<'a>> {
        self.tokenstream.consume(
            &TokenType::LeftParen,
            ParserErrorContext::ExpectedLeftParenAfterFor,
        )?;
        // We don't explicitly use the None value. So rust thinks it shouldn't be assigned
        #[allow(unused_assignments)]
        let mut initializer = None;
        if self.tokenstream.match_l(&[TokenType::Var])? {
            initializer = Some(self.var_declaration()?);
        } else {
            initializer = Some(self.expression_statement()?);
        }

        let mut condition = None;
        if !self.tokenstream.check(&TokenType::Semicolon)? {
            condition = Some(self.expression()?);
        }
        self.tokenstream.consume(
            &TokenType::Semicolon,
            ParserErrorContext::ExpectedSemicolonAfterLoopCondition,
        )?;

        let mut increment = None;
        if !self.tokenstream.check(&TokenType::RightParen)? {
            increment = Some(self.expression()?);
        }
        self.tokenstream.consume(
            &TokenType::RightParen,
            ParserErrorContext::ExpectedRightParenAfterForClause,
        )?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block(StmtBlock::new(vec![
                body,
                Stmt::Expression(StmtExpression::new(increment)),
            ]));
        }

        if let Some(condition) = condition {
            body = Stmt::While(StmtWhile::new(condition, Box::new(body)));
        } else {
            body = Stmt::While(StmtWhile::new(
                Expr::Literal(ExprLiteral::new(LiteralValue::Bool(true))),
                Box::new(body),
            ));
        }

        if let Some(initializer) = initializer {
            body = Stmt::Block(StmtBlock::new(vec![initializer, body]));
        }

        Ok(body)
    }

    fn expression_statement(&mut self) -> Result<Stmt<'a>, ParserError<'a>> {
        let value = self.expression()?;
        self.tokenstream
            .consume(&TokenType::Semicolon, ParserErrorContext::ExpectedSemicolon)?;
        Ok(Stmt::Expression(StmtExpression::new(value)))
    }

    fn function(&mut self, kind: ParserErrorContext) -> Result<Stmt<'a>, ParserError<'a>> {
        let name = self.tokenstream.consume(&TokenType::Ident, kind)?;
        self.tokenstream.consume(
            &TokenType::LeftParen,
            ParserErrorContext::ExpectedLeftParenAfterFunctionName,
        )?;

        let mut parameters = vec![];
        if !self.tokenstream.check(&TokenType::RightParen)? {
            if parameters.len() >= 255 {
                let token = self.tokenstream.peek()?;
                let err = ParserError::TooManyFunctionParameters { token: *token };
                eprintln!("{err:?}");
            }
            parameters.push(
                self.tokenstream
                    .consume(&TokenType::Ident, ParserErrorContext::ExpectedParameterName)?,
            );

            while self.tokenstream.match_l(&[TokenType::Comma])? {
                if parameters.len() >= 255 {
                    let token = self.tokenstream.peek()?;
                    let err = ParserError::TooManyFunctionParameters { token: *token };
                    eprintln!("{err:?}");
                }
                parameters.push(
                    self.tokenstream
                        .consume(&TokenType::Ident, ParserErrorContext::ExpectedParameterName)?,
                );
            }
        }
        self.tokenstream.consume(
            &TokenType::RightParen,
            ParserErrorContext::ExpectedRightParenAfterParameters,
        )?;

        self.tokenstream.consume(
            &TokenType::LeftBrace,
            ParserErrorContext::ExpectedLeftBraceBeforeFunctionBody,
        )?;
        let body = self.block()?;

        Ok(Stmt::Function(StmtFunction::new(name, parameters, body)))
    }

    fn expression(&mut self) -> Result<Expr<'a>, ParserError<'a>> {
        self.assignment()
    }

    fn call(&mut self) -> Result<Expr<'a>, ParserError<'a>> {
        let mut expr = self.primary()?;

        loop {
            if self.tokenstream.match_l(&[TokenType::LeftParen])? {
                expr = self.finish_call(expr)?;
            } else if self.tokenstream.match_l(&[TokenType::Dot])? {
                let name = self.tokenstream.consume(
                    &TokenType::Ident,
                    ParserErrorContext::ExpectedPropertyNameAfterDot,
                )?;
                expr = Expr::Get(ExprGet::new(Box::new(expr), name));
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr<'a>) -> Result<Expr<'a>, ParserError<'a>> {
        let mut arguments = vec![];

        if !self.tokenstream.check(&TokenType::RightParen)? {
            if arguments.len() >= 255 {
                let token = self.tokenstream.peek()?;
                let err = ParserError::TooManyFunctionArguments { token: *token };
                eprintln!("{err:?}");
            }
            arguments.push(self.expression()?);

            while self.tokenstream.match_l(&[TokenType::Comma])? {
                if arguments.len() >= 255 {
                    let token = self.tokenstream.peek()?;
                    let err = ParserError::TooManyFunctionArguments { token: *token };
                    eprintln!("{err:?}");
                }
                arguments.push(self.expression()?);
            }
        }

        let paren = self.tokenstream.consume(
            &TokenType::RightParen,
            ParserErrorContext::ExpectedRightParenAfterArguments,
        )?;

        Ok(Expr::Call(ExprCall::new(
            Box::new(callee),
            paren,
            arguments,
        )))
    }

    fn assignment(&mut self) -> Result<Expr<'a>, ParserError<'a>> {
        let expr = self.or()?;

        let operators = [TokenType::Equal];

        if self.tokenstream.match_l(&operators)? {
            let equals = self.tokenstream.previous()?;
            let value = self.assignment()?;

            if let Expr::Variable(var) = &expr {
                let name = var.name;
                return Ok(Expr::Assign(ExprAssign::new(name, Box::new(value))));
            } else if let Expr::Get(get) = expr {
                return Ok(Expr::Set(ExprSet::new(
                    get.object,
                    get.name,
                    Box::new(value),
                )));
            }

            return Err(ParserError::InvalidAssignmentTarget { token: equals });
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr<'a>, ParserError<'a>> {
        let mut expr = self.and()?;

        while self.tokenstream.match_l(&[TokenType::Or])? {
            let operator = self.tokenstream.previous()?;
            let right = self.and()?;
            expr = Expr::Logical(ExprLogical::new(Box::new(expr), operator, Box::new(right)));
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr<'a>, ParserError<'a>> {
        let mut expr = self.equality()?;

        while self.tokenstream.match_l(&[TokenType::And])? {
            let operator = self.tokenstream.previous()?;
            let right = self.equality()?;
            expr = Expr::Logical(ExprLogical::new(Box::new(expr), operator, Box::new(right)));
        }

        Ok(expr)
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

        self.call()
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
                self.tokenstream.consume(
                    &TokenType::RightParen,
                    ParserErrorContext::UnmatchedParanthesis,
                )?;
                return Ok(Expr::Grouping(ExprGrouping::new(Box::new(expr))));
            }
            TokenType::This => Ok(Expr::This(ExprThis::new(token))),
            TokenType::Ident => Ok(Expr::Variable(ExprVariable::new(token))),
            _ => Err(ParserError::UnexpectedToken { token }),
        }
    }

    fn synchronize(&mut self) -> Result<(), ParserError<'a>> {
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

        if let Ok(stmts) = parser.parse() {
            let left = Expr::Literal(ExprLiteral::new(LiteralValue::F64(2.0)));

            let span = Span { begin: 2, end: 3 };
            let operator = Token::new(TokenType::Plus, "+", 1, span);

            let right = Expr::Literal(ExprLiteral::new(LiteralValue::F64(3.0)));
            let stmt = vec![Stmt::Expression(StmtExpression::new(Expr::Binary(
                ExprBinary::new(Box::new(left), operator, Box::new(right)),
            )))];

            assert_eq!(stmt, stmts);
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
