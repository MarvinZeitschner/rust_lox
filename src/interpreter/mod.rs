pub mod callable;
pub mod class;
pub mod environment;
pub mod error;
pub mod native_fun;
pub mod resolver;
pub mod value;

use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use callable::LoxFunction;
use class::LoxClass;
use environment::Environment;
use error::{ClassError, Return, RuntimeError};
use native_fun::clock::Clock;
use value::Value;

use crate::{
    ast::*,
    lex::{Token, TokenType},
};

#[derive(Clone)]
pub struct Interpreter<'a> {
    environment: *mut Environment<'a>,
    globals: Box<Environment<'a>>,
    locals: HashMap<Expr<'a>, usize>,
}

impl<'a, 'b: 'a> Interpreter<'a> {
    pub fn new(locals: HashMap<Expr<'a>, usize>) -> Self {
        let mut globals = Box::new(Environment::new(None));
        globals.define("clock", Some(Value::Callable(Rc::new(Clock::new()))));

        let globals_ptr = &mut *globals as *mut Environment;

        Interpreter {
            globals,
            environment: globals_ptr,
            locals,
        }
    }

    fn get_mut_environment(&mut self) -> &mut Environment<'a> {
        unsafe { &mut *self.environment }
    }

    #[allow(dead_code)]
    fn get_environment(&self) -> &Environment<'a> {
        unsafe { &*self.environment }
    }

    fn get_ptr_environment(&mut self) -> *mut Environment<'a> {
        self.environment
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

    fn lookup_variable(
        &mut self,
        name: Token<'a>,
        expr: &Expr<'a>,
    ) -> Result<Value<'a>, RuntimeError<'a>> {
        let distance = self.locals.get(expr);
        match distance {
            Some(&d) => Ok(self.get_mut_environment().get_at(d, name.lexeme)),
            None => self.globals.get(name),
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

    fn visit_set(&mut self, node: &'b ExprSet<'a>) -> Self::Output {
        let object = self.evaluate(&node.object)?;

        let Value::Instance(instance) = object else {
            return Err(RuntimeError::ClassError(
                ClassError::InvalidPropertyAccess { token: node.name },
            ));
        };

        let value = self.evaluate(&node.value)?;
        // TODO: Clone
        instance.borrow_mut().set(node.name, value.clone());
        Ok(value)
    }

    fn visit_super(&mut self, node: &'b ExprSuper<'a>) -> Self::Output {
        let distance = self
            .locals
            .get(&Expr::Super(node.clone()))
            .cloned()
            .unwrap();

        let superclass = self.get_mut_environment().get_at(distance, "super");
        let object = self.get_mut_environment().get_at(distance - 1, "this");

        let superclass = match superclass {
            Value::Callable(callable) => callable.clone_as_class().ok_or(
                RuntimeError::ClassError(ClassError::SuperclassNotAClass {
                    token: node.keyword,
                }),
            )?,
            _ => {
                return Err(RuntimeError::ClassError(ClassError::SuperclassNotAClass {
                    token: node.keyword,
                }))
            }
        };
        let object = match object {
            Value::Instance(instance) => instance,
            _ => {
                return Err(RuntimeError::ClassError(ClassError::SuperclassNotAClass {
                    token: node.keyword,
                }))
            }
        };

        let method = superclass
            .find_method(node.method.lexeme)
            .ok_or(RuntimeError::ClassError(ClassError::UndefinedProperty {
                token: node.method,
            }))?;

        let a = method.bind_rc(object);
        Ok(Value::Callable(Rc::new(a)))
    }

    fn visit_this(&mut self, node: &'b ExprThis<'a>) -> Self::Output {
        // TODO: Clone
        self.lookup_variable(node.keyword, &Expr::This(node.clone()))
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

    fn visit_get(&mut self, node: &'b ExprGet<'a>) -> Self::Output {
        let object = self.evaluate(&node.object)?;

        if let Value::Instance(instance) = object {
            return instance.borrow().get(node.name);
        }

        Err(RuntimeError::ClassError(
            ClassError::InvalidPropertyAccess { token: node.name },
        ))
    }

    fn visit_assign(&mut self, node: &ExprAssign<'a>) -> Self::Output {
        let value = self.evaluate(&node.value)?;

        let distance = self.locals.get(&Expr::Assign(node.clone())).cloned();
        match distance {
            Some(d) => {
                self.get_mut_environment()
                    .assign_at(d, node.name, value.clone());
            }
            None => self.globals.assign(node.name, value.clone())?,
        }

        Ok(value)
    }

    fn visit_variable(&mut self, node: &ExprVariable<'a>) -> Self::Output {
        self.lookup_variable(node.name, &Expr::Variable(node.clone()))
    }
}

impl<'a, 'b: 'a> StmtVisitor<'a, 'b> for Interpreter<'a> {
    type Output = Result<(), RuntimeError<'a>>;

    fn visit_block(&mut self, node: &'b StmtBlock<'a>) -> Self::Output {
        self.execute_block(&node.statements, Environment::new(Some(self.environment)))?;
        Ok(())
    }

    fn visit_class(&mut self, node: &'b StmtClass<'a>) -> Self::Output {
        let mut superclass = None;
        let mut superclass_value = None;
        if let Some(sc) = &node.superclass {
            superclass_value = Some(self.evaluate(sc)?);
            superclass = match superclass_value.as_ref().unwrap() {
                Value::Callable(callable) => {
                    let class = callable.clone_as_class().ok_or(RuntimeError::ClassError(
                        ClassError::SuperclassNotAClass { token: node.name },
                    ))?;
                    Some(class)
                }
                _ => None,
            };
        }

        self.get_mut_environment().define(node.name.lexeme, None);

        if node.superclass.is_some() {
            self.environment = Box::into_raw(Box::new(Environment::new(Some(self.environment))));
            self.get_mut_environment().define("super", superclass_value);
        }

        let mut methods = HashMap::new();
        node.methods.iter().for_each(|method| {
            let function = LoxFunction::new(
                method,
                self.get_ptr_environment(),
                method.name.lexeme.eq("init"),
            );
            methods.insert(method.name.lexeme, function);
        });

        let class = LoxClass::new(node.name.lexeme, superclass, methods);

        if node.superclass.is_some() {
            self.environment = self.get_mut_environment().enclosing.unwrap();
        }

        self.get_mut_environment()
            .assign(node.name, Value::Callable(Rc::new(class)))?;

        Ok(())
    }

    fn visit_expression(&mut self, node: &StmtExpression<'a>) -> Self::Output {
        self.evaluate(&node.expr)?;
        Ok(())
    }

    fn visit_function(&mut self, node: &'b StmtFunction<'a>) -> Self::Output {
        let function = LoxFunction::new(node, self.get_ptr_environment(), false);

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
        let mut value = Value::Nil;

        if node.value.is_some() {
            value = self.evaluate(node.value.as_ref().unwrap())?;
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

    fn setup() -> Interpreter<'static> {
        let locals = HashMap::new();
        Interpreter::new(locals)
    }

    #[test]
    fn literal() {
        let mut interpreter = setup();

        let expr = Expr::Literal(ExprLiteral::new(LiteralValue::F64(1.0)));
        let result = interpreter.evaluate(&expr).unwrap();

        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn grouping() {
        let mut interpreter = setup();

        let expr = Expr::Grouping(ExprGrouping::new(Box::new(Expr::Literal(
            ExprLiteral::new(LiteralValue::F64(1.0)),
        ))));
        let result = interpreter.evaluate(&expr).unwrap();

        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn unary() {
        let mut interpreter = setup();

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
        let mut interpreter = setup();

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
