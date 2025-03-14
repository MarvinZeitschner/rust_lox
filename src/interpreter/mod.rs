pub mod callable;
pub mod environment;
pub mod error;
pub mod native_fun;
pub mod value;

use std::{collections::VecDeque, rc::Rc};

use callable::LoxFunction;
use environment::Environment;
use error::{Return, RuntimeError};
use native_fun::clock::Clock;
use value::Value;

use crate::{
    ast::*,
    lex::{Token, TokenType},
};

pub struct Interpreter<'a> {
    environment: *mut Environment<'a>,
    // rust sees globals as unused, but its actually used for native functions. A reference to a
    // raw ptr of globals is safed in the environment
    #[allow(dead_code)]
    globals: Box<Environment<'a>>,
}

impl<'a> Default for Interpreter<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, 'b: 'a> Interpreter<'a> {
    pub fn new() -> Self {
        let mut globals = Box::new(Environment::new(None));
        globals.define("clock", Some(Value::Callable(Rc::new(Clock::new()))));

        let globals_ptr = &mut *globals as *mut Environment;

        Interpreter {
            globals,
            environment: globals_ptr,
        }
    }

    fn get_mut_environment(&mut self) -> &mut Environment<'a> {
        unsafe { &mut *self.environment }
    }

    fn get_environment(&self) -> &Environment<'a> {
        unsafe { &*self.environment }
    }

    fn get_mut_globals(&mut self) -> &mut Environment<'a> {
        &mut self.globals
    }

    pub fn interpret(&mut self, stmts: &'b [Stmt<'a>]) -> Result<(), RuntimeError<'a>> {
        stmts.iter().try_for_each(|stmt| self.execute(stmt))?;
        Ok(())
    }

    fn execute(&mut self, stmt: &'b Stmt<'a>) -> Result<(), RuntimeError<'a>> {
        stmt.accept(self)
    }

    fn execute_block(
        &mut self,
        statements: &'b [Stmt<'a>],
        environment: Environment<'a>,
    ) -> Result<(), RuntimeError<'a>> {
        // I will leaves this here as it was a cool approach before the need of Rc's and now raw
        // pointers
        // std::mem::swap(&mut self.environment, &mut environment);
        //
        // let result = statements.iter().try_for_each(|stmt| self.execute(stmt));
        //
        // if let Some(enclosing) = self.environment.enclosing.take() {
        //     self.environment = *enclosing;
        // }
        //
        // result

        let prev = self.environment;

        let env_ptr = Box::into_raw(Box::new(environment));

        self.environment = env_ptr;

        let result = statements.iter().try_for_each(|stmt| self.execute(stmt));

        let _drop = Box::from(env_ptr);
        self.environment = prev;

        result
    }

    fn evaluate(&mut self, expr: &Expr<'a>) -> Result<Value<'a>, RuntimeError<'a>> {
        expr.accept(self)
    }

    fn check_number_operand(
        &mut self,
        value: &Value,
        operator: Token<'a>,
    ) -> Result<(), RuntimeError<'a>> {
        match value {
            Value::Number(_) => Ok(()),
            _ => Err(RuntimeError::NumberOperand { operator }),
        }
    }

    fn check_number_operands(
        &mut self,
        left: &Value,
        right: &Value,
        operator: Token<'a>,
    ) -> Result<(), RuntimeError<'a>> {
        match (left, right) {
            (Value::Number(_), Value::Number(_)) => Ok(()),
            _ => Err(RuntimeError::MutlipleNumberOperands { operator }),
        }
    }
}

impl<'a, 'b> ExprVisitor<'a, 'b> for Interpreter<'a> {
    type Output = Result<Value<'a>, RuntimeError<'a>>;

    fn visit_literal(&mut self, node: &ExprLiteral) -> Self::Output {
        Ok(node.value.clone().into())
    }

    fn visit_grouping(&mut self, node: &ExprGrouping<'a>) -> Self::Output {
        self.evaluate(&node.value)
    }

    fn visit_logical(&mut self, node: &ExprLogical<'a>) -> Self::Output {
        let left = self.evaluate(&node.left)?;

        if (node.operator.kind == TokenType::Or && left.is_truthy())
            || (node.operator.kind == TokenType::And && !left.is_truthy())
        {
            return Ok(left);
        }

        self.evaluate(&node.right)
    }

    fn visit_unary(&mut self, node: &ExprUnary<'a>) -> Self::Output {
        let operator = node.operator;
        let right = self.evaluate(&node.value)?;

        match node.operator.kind {
            TokenType::Minus => {
                self.check_number_operand(&right, operator)?;
                Ok(-right)
            }
            TokenType::Bang => Ok(!right),
            _ => Err(RuntimeError::NumberOperand { operator }),
        }
    }

    fn visit_binary(&mut self, node: &ExprBinary<'a>) -> Self::Output {
        let operator = node.operator;
        let left = self.evaluate(&node.left)?;
        let right = self.evaluate(&node.right)?;

        match node.operator.kind {
            TokenType::Minus => {
                self.check_number_operands(&left, &right, operator)?;
                Ok(left - right)
            }
            TokenType::Slash => {
                self.check_number_operands(&left, &right, operator)?;
                Ok(left / right)
            }
            TokenType::Star => {
                self.check_number_operands(&left, &right, operator)?;
                Ok(left * right)
            }
            TokenType::Plus => {
                if let (Value::String(_), Value::String(_)) = (&left, &right) {
                    return Ok(left + right);
                }
                if let (Value::Number(_), Value::Number(_)) = (&left, &right) {
                    return Ok(left + right);
                }
                Err(RuntimeError::NumberOrStringOperands { operator })
            }
            TokenType::Greater => {
                self.check_number_operands(&left, &right, operator)?;
                Ok(Value::Boolean(left > right))
            }
            TokenType::Less => {
                self.check_number_operands(&left, &right, operator)?;
                Ok(Value::Boolean(left < right))
            }
            TokenType::GreaterEqual => {
                self.check_number_operands(&left, &right, operator)?;
                Ok(Value::Boolean(left >= right))
            }
            TokenType::LessEqual => {
                self.check_number_operands(&left, &right, operator)?;
                Ok(Value::Boolean(left <= right))
            }
            TokenType::EqualEqual => Ok(Value::Boolean(left == right)),
            TokenType::BangEqual => Ok(Value::Boolean(left != right)),
            _ => Ok(Value::Nil),
        }
    }

    fn visit_call(&mut self, node: &ExprCall<'a>) -> Self::Output {
        let callee = self.evaluate(&node.callee)?;

        let arguments: VecDeque<Value<'a>> = node
            .arguments
            .iter()
            .map(|argument| self.evaluate(argument))
            .collect::<Result<VecDeque<_>, _>>()?;

        let Value::Callable(function) = callee else {
            return Err(RuntimeError::NotCallable { token: node.paren });
        };

        if arguments.len() != function.arity() {
            return Err(RuntimeError::ArgumentCount {
                token: node.paren,
                expected_arity: function.arity(),
                given_len: arguments.len(),
            });
        }

        function.call(self, arguments)
    }

    fn visit_assign(&mut self, node: &ExprAssign<'a>) -> Self::Output {
        let value = self.evaluate(&node.value)?;
        self.get_mut_environment()
            .assign(node.name, value.clone())?;
        Ok(value)
    }

    fn visit_variable(&mut self, node: &ExprVariable<'a>) -> Self::Output {
        self.get_environment().get(node.name)
    }
}

impl<'a, 'b: 'a> StmtVisitor<'a, 'b> for Interpreter<'a> {
    type Output = Result<(), RuntimeError<'a>>;

    fn visit_block(&mut self, node: &'b StmtBlock<'a>) -> Self::Output {
        self.execute_block(&node.statements, Environment::new(Some(self.environment)))?;
        Ok(())
    }

    fn visit_expression(&mut self, node: &StmtExpression<'a>) -> Self::Output {
        self.evaluate(&node.expr)?;
        Ok(())
    }

    fn visit_function(&mut self, node: &'b StmtFunction<'a>) -> Self::Output {
        let function = LoxFunction::new(node);

        self.get_mut_environment()
            .define(node.name.lexeme, Some(Value::Callable(Rc::new(function))));

        Ok(())
    }

    fn visit_if(&mut self, node: &'b StmtIf<'a>) -> Self::Output {
        let condition = self.evaluate(&node.condition)?;
        if condition.is_truthy() {
            self.execute(&node.then_branch)?;
        } else if let Some(stmt) = &node.else_branch {
            self.execute(stmt)?;
        }

        Ok(())
    }

    fn visit_print(&mut self, node: &StmtPrint<'a>) -> Self::Output {
        let value = self.evaluate(&node.expr)?;
        println!("{}", value);
        Ok(())
    }

    fn visit_return(&mut self, node: &'b StmtReturn<'a>) -> Self::Output {
        let mut value = None;

        if node.value.is_some() {
            value = Some(self.evaluate(node.value.as_ref().unwrap())?);
        }

        Err(RuntimeError::Return(Return { value }))
    }

    fn visit_var(&mut self, node: &StmtVar<'a>) -> Self::Output {
        let mut value = None;
        if let Some(initializer) = &node.initializer {
            value = Some(self.evaluate(initializer)?);
        }
        self.get_mut_environment().define(node.name.lexeme, value);
        Ok(())
    }

    fn visit_while(&mut self, node: &'b StmtWhile<'a>) -> Self::Output {
        while self.evaluate(&node.condition)?.is_truthy() {
            self.execute(&node.body)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::lex::{Span, Token};

    use super::*;

    #[test]
    fn literal() {
        let mut interpreter = Interpreter::new();

        let expr = Expr::Literal(ExprLiteral::new(LiteralValue::F64(1.0)));
        let result = interpreter.evaluate(&expr).unwrap();

        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn grouping() {
        let mut interpreter = Interpreter::new();

        let expr = Expr::Grouping(ExprGrouping::new(Box::new(Expr::Literal(
            ExprLiteral::new(LiteralValue::F64(1.0)),
        ))));
        let result = interpreter.evaluate(&expr).unwrap();

        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn unary() {
        let mut interpreter = Interpreter::new();

        let span = Span { begin: 0, end: 1 };
        let token = Token::new(TokenType::Minus, "-", 1, span);
        let expr = Expr::Unary(ExprUnary::new(
            token,
            Box::new(Expr::Literal(ExprLiteral::new(LiteralValue::F64(1.0)))),
        ));
        let result = interpreter.evaluate(&expr).unwrap();

        assert_eq!(result, Value::Number(-1.0));
    }

    #[test]
    fn error() {
        let mut interpreter = Interpreter::new();

        let span = Span { begin: 0, end: 1 };
        let token = Token::new(TokenType::Minus, "-", 1, span);
        let expr = Expr::Unary(ExprUnary::new(
            token,
            Box::new(Expr::Literal(ExprLiteral::new(LiteralValue::String(
                "1".to_string(),
            )))),
        ));
        let result = interpreter.evaluate(&expr);

        assert_eq!(result, Err(RuntimeError::NumberOperand { operator: token }));
    }
}
