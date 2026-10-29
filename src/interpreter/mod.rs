pub mod callable;
pub mod class;
pub mod environment;
pub mod error;
pub mod native_fun;
pub mod resolver;
pub mod value;

use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    io::{self, Write},
    rc::Rc,
};

use callable::LoxFunction;
use class::{LoxClass, LoxInstance};
use environment::Environment;
use error::{ClassError, Return, RuntimeError};
use native_fun::clock::Clock;
use value::Value;

use crate::{
    ast::*,
    lex::{Token, TokenType},
};

pub struct Interpreter<'a> {
    environment: Rc<RefCell<Environment<'a>>>,
    globals: Rc<RefCell<Environment<'a>>>,
    output: Box<dyn Write>,
}

impl<'a> Interpreter<'a> {
    /// Creates an interpreter that prints to stdout.
    pub fn new() -> Self {
        Self::with_output(Box::new(io::stdout()))
    }

    /// Creates an interpreter that writes the output of `print` statements to `output`.
    pub fn with_output(output: Box<dyn Write>) -> Self {
        let globals = Rc::new(RefCell::new(Environment::new(None)));
        globals
            .borrow_mut()
            .define("clock", Some(Value::Callable(Rc::new(Clock::new()))));

        Interpreter {
            environment: globals.clone(),
            globals,
            output,
        }
    }

    pub fn interpret(&mut self, stmts: &'a [Stmt<'a>]) -> Result<(), RuntimeError<'a>> {
        stmts.iter().try_for_each(|stmt| self.execute(stmt))?;
        Ok(())
    }

    fn execute(&mut self, stmt: &'a Stmt<'a>) -> Result<(), RuntimeError<'a>> {
        match stmt {
            Stmt::Block(node) => self.visit_block(node),
            Stmt::Class(node) => self.visit_class(node),
            Stmt::Expression(node) => self.visit_expression(node),
            Stmt::Function(node) => self.visit_function(node),
            Stmt::If(node) => self.visit_if(node),
            Stmt::Print(node) => self.visit_print(node),
            Stmt::Return(node) => self.visit_return(node),
            Stmt::Var(node) => self.visit_var(node),
            Stmt::While(node) => self.visit_while(node),
        }
    }

    fn execute_block(
        &mut self,
        statements: &'a [Stmt<'a>],
        environment: Environment<'a>,
    ) -> Result<(), RuntimeError<'a>> {
        // The block's environment is dropped once nothing references it anymore. Closures
        // created inside the block keep it alive through their own Rc.
        let prev = std::mem::replace(&mut self.environment, Rc::new(RefCell::new(environment)));

        let result = statements.iter().try_for_each(|stmt| self.execute(stmt));

        self.environment = prev;

        result
    }

    fn evaluate(&mut self, expr: &Expr<'a>) -> Result<Value<'a>, RuntimeError<'a>> {
        match expr {
            Expr::Literal(node) => self.visit_literal(node),
            Expr::Grouping(node) => self.visit_grouping(node),
            Expr::Logical(node) => self.visit_logical(node),
            Expr::Set(node) => self.visit_set(node),
            Expr::Super(node) => self.visit_super(node),
            Expr::This(node) => self.visit_this(node),
            Expr::Unary(node) => self.visit_unary(node),
            Expr::Binary(node) => self.visit_binary(node),
            Expr::Call(node) => self.visit_call(node),
            Expr::Get(node) => self.visit_get(node),
            Expr::Assign(node) => self.visit_assign(node),
            Expr::Variable(node) => self.visit_variable(node),
        }
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

    /// `depth` is the resolver's result: `Some(d)` for a local `d` environments up, `None` for a global.
    fn lookup_variable(
        &mut self,
        name: Token<'a>,
        depth: Option<usize>,
    ) -> Result<Value<'a>, RuntimeError<'a>> {
        match depth {
            Some(d) => Ok(self.environment.borrow().get_at(d, name.lexeme)),
            None => self.globals.borrow().get(name),
        }
    }
}

/// Evaluation of the individual expression types, dispatched by `evaluate`
impl<'a> Interpreter<'a> {
    fn visit_literal(&mut self, node: &ExprLiteral) -> Result<Value<'a>, RuntimeError<'a>> {
        Ok(node.value.clone().into())
    }

    fn visit_grouping(&mut self, node: &ExprGrouping<'a>) -> Result<Value<'a>, RuntimeError<'a>> {
        self.evaluate(&node.value)
    }

    fn visit_logical(&mut self, node: &ExprLogical<'a>) -> Result<Value<'a>, RuntimeError<'a>> {
        let left = self.evaluate(&node.left)?;

        if (node.operator.kind == TokenType::Or && left.is_truthy())
            || (node.operator.kind == TokenType::And && !left.is_truthy())
        {
            return Ok(left);
        }

        self.evaluate(&node.right)
    }

    fn visit_set(&mut self, node: &ExprSet<'a>) -> Result<Value<'a>, RuntimeError<'a>> {
        let object = self.evaluate(&node.object)?;

        let Value::Instance(instance) = object else {
            return Err(RuntimeError::ClassError(ClassError::InvalidFieldAccess {
                token: node.name,
            }));
        };

        let value = self.evaluate(&node.value)?;
        // TODO: Clone
        instance.borrow_mut().set(node.name, value.clone());
        Ok(value)
    }

    fn visit_super(&mut self, node: &ExprSuper<'a>) -> Result<Value<'a>, RuntimeError<'a>> {
        // `super` is always local to a method, so the resolver always sets the depth
        let distance = node.depth.get().expect("resolver sets the depth of 'super'");

        let superclass = self.environment.borrow().get_at(distance, "super");
        let object = self.environment.borrow().get_at(distance - 1, "this");

        let superclass = match superclass {
            Value::Callable(callable) => callable.as_class().ok_or(
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

        Ok(Value::Callable(Rc::new(method.bind(object))))
    }

    fn visit_this(&mut self, node: &ExprThis<'a>) -> Result<Value<'a>, RuntimeError<'a>> {
        // TODO: Clone
        self.lookup_variable(node.keyword, node.depth.get())
    }

    fn visit_unary(&mut self, node: &ExprUnary<'a>) -> Result<Value<'a>, RuntimeError<'a>> {
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

    fn visit_binary(&mut self, node: &ExprBinary<'a>) -> Result<Value<'a>, RuntimeError<'a>> {
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

    fn visit_call(&mut self, node: &ExprCall<'a>) -> Result<Value<'a>, RuntimeError<'a>> {
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

    fn visit_get(&mut self, node: &ExprGet<'a>) -> Result<Value<'a>, RuntimeError<'a>> {
        let object = self.evaluate(&node.object)?;

        if let Value::Instance(instance) = object {
            return LoxInstance::get(&instance, node.name);
        }

        Err(RuntimeError::ClassError(
            ClassError::InvalidPropertyAccess { token: node.name },
        ))
    }

    fn visit_assign(&mut self, node: &ExprAssign<'a>) -> Result<Value<'a>, RuntimeError<'a>> {
        let value = self.evaluate(&node.value)?;

        match node.depth.get() {
            Some(d) => {
                self.environment
                    .borrow_mut()
                    .assign_at(d, node.name, value.clone());
            }
            None => self.globals.borrow_mut().assign(node.name, value.clone())?,
        }

        Ok(value)
    }

    fn visit_variable(&mut self, node: &ExprVariable<'a>) -> Result<Value<'a>, RuntimeError<'a>> {
        self.lookup_variable(node.name, node.depth.get())
    }
}

/// Execution of the individual statement types, dispatched by `execute`.
/// Statements borrow the AST for `'a` because functions keep a reference to their declaration.
impl<'a> Interpreter<'a> {
    fn visit_block(&mut self, node: &'a StmtBlock<'a>) -> Result<(), RuntimeError<'a>> {
        self.execute_block(
            &node.statements,
            Environment::new(Some(self.environment.clone())),
        )?;
        Ok(())
    }

    fn visit_class(&mut self, node: &'a StmtClass<'a>) -> Result<(), RuntimeError<'a>> {
        let mut superclass = None;
        let mut superclass_value = None;
        if let Some(sc) = &node.superclass {
            superclass_value = Some(self.evaluate(sc)?);
            superclass = match superclass_value.as_ref().unwrap() {
                Value::Callable(callable) => {
                    let class = callable.clone().as_class().ok_or(RuntimeError::ClassError(
                        ClassError::SuperclassNotAClass { token: node.name },
                    ))?;
                    Some(class)
                }
                _ => {
                    return Err(RuntimeError::ClassError(
                        ClassError::SuperclassNotAClass { token: node.name },
                    ))
                }
            };
        }

        self.environment.borrow_mut().define(node.name.lexeme, None);

        if node.superclass.is_some() {
            let mut environment = Environment::new(Some(self.environment.clone()));
            environment.define("super", superclass_value);
            self.environment = Rc::new(RefCell::new(environment));
        }

        let mut methods = HashMap::new();
        node.methods.iter().for_each(|method| {
            let function = LoxFunction::new(
                method,
                self.environment.clone(),
                method.name.lexeme.eq("init"),
            );
            methods.insert(method.name.lexeme, function);
        });

        let class = LoxClass::new(node.name.lexeme, superclass, methods);

        if node.superclass.is_some() {
            let enclosing = self.environment.borrow().enclosing.clone().unwrap();
            self.environment = enclosing;
        }

        self.environment
            .borrow_mut()
            .assign(node.name, Value::Callable(Rc::new(class)))?;

        Ok(())
    }

    fn visit_expression(&mut self, node: &StmtExpression<'a>) -> Result<(), RuntimeError<'a>> {
        self.evaluate(&node.expr)?;
        Ok(())
    }

    fn visit_function(&mut self, node: &'a StmtFunction<'a>) -> Result<(), RuntimeError<'a>> {
        let function = LoxFunction::new(node, self.environment.clone(), false);

        self.environment
            .borrow_mut()
            .define(node.name.lexeme, Some(Value::Callable(Rc::new(function))));

        Ok(())
    }

    fn visit_if(&mut self, node: &'a StmtIf<'a>) -> Result<(), RuntimeError<'a>> {
        let condition = self.evaluate(&node.condition)?;
        if condition.is_truthy() {
            self.execute(&node.then_branch)?;
        } else if let Some(stmt) = &node.else_branch {
            self.execute(stmt)?;
        }

        Ok(())
    }

    fn visit_print(&mut self, node: &StmtPrint<'a>) -> Result<(), RuntimeError<'a>> {
        let value = self.evaluate(&node.expr)?;
        writeln!(self.output, "{}", value).map_err(|e| RuntimeError::Output(e.to_string()))?;
        Ok(())
    }

    fn visit_return(&mut self, node: &'a StmtReturn<'a>) -> Result<(), RuntimeError<'a>> {
        let mut value = Value::Nil;

        if node.value.is_some() {
            value = self.evaluate(node.value.as_ref().unwrap())?;
        }

        Err(RuntimeError::Return(Return { value }))
    }

    fn visit_var(&mut self, node: &StmtVar<'a>) -> Result<(), RuntimeError<'a>> {
        let mut value = None;
        if let Some(initializer) = &node.initializer {
            value = Some(self.evaluate(initializer)?);
        }
        self.environment.borrow_mut().define(node.name.lexeme, value);
        Ok(())
    }

    fn visit_while(&mut self, node: &'a StmtWhile<'a>) -> Result<(), RuntimeError<'a>> {
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
        Interpreter::new()
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
