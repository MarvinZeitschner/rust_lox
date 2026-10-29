use std::{cell::Cell, collections::HashMap};

use crate::{ast::*, lex::Token};

use super::error::ResolverError;

#[derive(Default, Copy, Clone, PartialEq)]
pub enum FunctionType {
    #[default]
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Default, Copy, Clone, PartialEq)]
pub enum ClassType {
    #[default]
    None,
    Class,
    Subclass,
}

/// A local variable in a scope of the resolver.
#[derive(Clone, Copy)]
struct Declaration {
    /// Position in the environment at runtime. Variables get their slots in declaration order,
    /// which is the same order in which the interpreter creates them.
    slot: usize,
    /// `false` while the variable's own initializer is being resolved
    defined: bool,
}

#[derive(Default)]
pub struct Resolver<'a> {
    scopes: Vec<HashMap<&'a str, Declaration>>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl<'a> Resolver<'a> {
    pub fn new() -> Self {
        Self {
            scopes: vec![],
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    /// Resolves all statements and collects one error per failing top-level statement.
    pub fn resolve(&mut self, stmts: &[Stmt<'a>]) -> Result<(), Vec<ResolverError<'a>>> {
        let mut errors = vec![];
        for stmt in stmts {
            if let Err(err) = self.resolve_stmt(stmt) {
                errors.push(err);
                // The error aborted the statement midway, so return to the top-level state
                self.scopes.clear();
                self.current_function = FunctionType::None;
                self.current_class = ClassType::None;
            }
        }

        match errors.is_empty() {
            true => Ok(()),
            false => Err(errors),
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt<'a>) -> Result<(), ResolverError<'a>> {
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

    fn resolve_stmts(&mut self, stmts: &[Stmt<'a>]) -> Result<(), ResolverError<'a>> {
        stmts.iter().try_for_each(|stmt| self.resolve_stmt(stmt))
    }

    fn resolve_expr(&mut self, expr: &Expr<'a>) -> Result<(), ResolverError<'a>> {
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

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token<'a>) -> Result<(), ResolverError<'a>> {
        if self.scopes.is_empty() {
            return Ok(());
        }

        let scope = self.scopes.last_mut().unwrap();

        if scope.contains_key(name.lexeme) {
            Err(ResolverError::SameNameVariableInLocalScope { token: *name })
        } else {
            let slot = scope.len();
            scope.insert(
                name.lexeme,
                Declaration {
                    slot,
                    defined: false,
                },
            );
            Ok(())
        }
    }

    fn define(&mut self, name: &Token<'a>) {
        if let Some(declaration) = self
            .scopes
            .last_mut()
            .and_then(|scope| scope.get_mut(name.lexeme))
        {
            declaration.defined = true;
        }
    }

    /// Opens a scope that only contains `name`, like the ones for `this` and `super`.
    fn begin_scope_with(&mut self, name: &'a str) {
        let declaration = Declaration {
            slot: 0,
            defined: true,
        };
        self.scopes.push(HashMap::from([(name, declaration)]));
    }

    /// Stores in `local` where `name` is declared. Globals are left at `None`.
    fn resolve_local(&mut self, local: &Cell<Option<Local>>, name: Token<'a>) {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if let Some(declaration) = scope.get(name.lexeme) {
                local.set(Some(Local {
                    depth: self.scopes.len() - 1 - i,
                    slot: declaration.slot,
                }));
                return;
            }
        }
    }

    fn resolve_function(
        &mut self,
        function: &StmtFunction<'a>,
        fn_type: FunctionType,
    ) -> Result<(), ResolverError<'a>> {
        let enclosing_function = self.current_function;
        self.current_function = fn_type;

        self.begin_scope();
        function
            .params
            .iter()
            .try_for_each(|param| -> Result<(), ResolverError<'a>> {
                self.declare(param)?;
                self.define(param);
                Ok(())
            })?;
        self.resolve_stmts(&function.body)?;
        self.end_scope();

        self.current_function = enclosing_function;
        Ok(())
    }
}

/// Resolution of the individual expression types, dispatched by `resolve_expr`
impl<'a> Resolver<'a> {
    fn visit_literal(&mut self, _node: &ExprLiteral) -> Result<(), ResolverError<'a>> {
        Ok(())
    }

    fn visit_grouping(&mut self, node: &ExprGrouping<'a>) -> Result<(), ResolverError<'a>> {
        self.resolve_expr(&node.value)?;
        Ok(())
    }

    fn visit_logical(&mut self, node: &ExprLogical<'a>) -> Result<(), ResolverError<'a>> {
        self.resolve_expr(&node.left)?;
        self.resolve_expr(&node.right)?;
        Ok(())
    }

    fn visit_set(&mut self, node: &ExprSet<'a>) -> Result<(), ResolverError<'a>> {
        self.resolve_expr(&node.value)?;
        self.resolve_expr(&node.object)?;
        Ok(())
    }

    fn visit_super(&mut self, node: &ExprSuper<'a>) -> Result<(), ResolverError<'a>> {
        if self.current_class == ClassType::None {
            return Err(ResolverError::SuperOutsideClass {
                token: node.keyword,
            });
        } else if self.current_class != ClassType::Subclass {
            return Err(ResolverError::SuperInClassWithoutSuperclass {
                token: node.keyword,
            });
        }
        self.resolve_local(&node.local, node.keyword);
        Ok(())
    }

    fn visit_this(&mut self, node: &ExprThis<'a>) -> Result<(), ResolverError<'a>> {
        if self.current_class == ClassType::None {
            return Err(ResolverError::ThisOutsideClass {
                token: node.keyword,
            });
        }

        self.resolve_local(&node.local, node.keyword);
        Ok(())
    }

    fn visit_unary(&mut self, node: &ExprUnary<'a>) -> Result<(), ResolverError<'a>> {
        self.resolve_expr(&node.value)?;
        Ok(())
    }

    fn visit_binary(&mut self, node: &ExprBinary<'a>) -> Result<(), ResolverError<'a>> {
        self.resolve_expr(&node.left)?;
        self.resolve_expr(&node.right)?;
        Ok(())
    }

    fn visit_call(&mut self, node: &ExprCall<'a>) -> Result<(), ResolverError<'a>> {
        self.resolve_expr(&node.callee)?;
        node.arguments
            .iter()
            .try_for_each(|arg| self.resolve_expr(arg))
    }

    fn visit_get(&mut self, node: &ExprGet<'a>) -> Result<(), ResolverError<'a>> {
        self.resolve_expr(&node.object)?;
        Ok(())
    }

    fn visit_assign(&mut self, node: &ExprAssign<'a>) -> Result<(), ResolverError<'a>> {
        self.resolve_expr(&node.value)?;
        self.resolve_local(&node.local, node.name);
        Ok(())
    }

    fn visit_variable(&mut self, node: &ExprVariable<'a>) -> Result<(), ResolverError<'a>> {
        if !self.scopes.is_empty() {
            let scope = self.scopes.last().unwrap();

            if let Some(declaration) = scope.get(node.name.lexeme) {
                if !declaration.defined {
                    return Err(ResolverError::VariableInOwnInitializer { token: node.name });
                }
            }
        }

        self.resolve_local(&node.local, node.name);

        Ok(())
    }
}

/// Resolution of the individual statement types, dispatched by `resolve_stmt`
impl<'a> Resolver<'a> {
    fn visit_block(&mut self, node: &StmtBlock<'a>) -> Result<(), ResolverError<'a>> {
        self.begin_scope();
        self.resolve_stmts(&node.statements)?;
        self.end_scope();
        Ok(())
    }

    fn visit_class(&mut self, node: &StmtClass<'a>) -> Result<(), ResolverError<'a>> {
        let enclosing_class = self.current_class;
        self.current_class = ClassType::Class;
        self.declare(&node.name)?;
        self.define(&node.name);

        if let Some(superclass) = &node.superclass {
            if let Expr::Variable(superclass) = superclass {
                if superclass.name.lexeme == node.name.lexeme {
                    return Err(ResolverError::InheritanceCycle {
                        token: superclass.name,
                    });
                }
            } else {
                panic!("Internal error");
            }
            self.current_class = ClassType::Class;
            self.resolve_expr(superclass)?;
        }

        if node.superclass.is_some() {
            self.current_class = ClassType::Subclass;
            self.begin_scope_with("super");
        }

        self.begin_scope_with("this");

        node.methods.iter().try_for_each(|method| {
            let mut declaration = FunctionType::Method;
            if method.name.lexeme == "init" {
                declaration = FunctionType::Initializer;
            }
            self.resolve_function(method, declaration)
        })?;

        self.end_scope();

        if node.superclass.is_some() {
            self.end_scope();
        }

        self.current_class = enclosing_class;

        Ok(())
    }

    fn visit_expression(&mut self, node: &StmtExpression<'a>) -> Result<(), ResolverError<'a>> {
        self.resolve_expr(&node.expr)
    }

    fn visit_function(&mut self, node: &StmtFunction<'a>) -> Result<(), ResolverError<'a>> {
        self.declare(&node.name)?;
        self.define(&node.name);

        self.resolve_function(node, FunctionType::Function)?;
        Ok(())
    }

    fn visit_if(&mut self, node: &StmtIf<'a>) -> Result<(), ResolverError<'a>> {
        self.resolve_expr(&node.condition)?;
        self.resolve_stmt(&node.then_branch)?;
        if let Some(else_branch) = &node.else_branch {
            self.resolve_stmt(else_branch)?;
        }
        Ok(())
    }

    fn visit_print(&mut self, node: &StmtPrint<'a>) -> Result<(), ResolverError<'a>> {
        self.resolve_expr(&node.expr)
    }

    fn visit_return(&mut self, node: &StmtReturn<'a>) -> Result<(), ResolverError<'a>> {
        if self.current_function == FunctionType::None {
            return Err(ResolverError::TopLevelReturn {
                token: node.keyword,
            });
        }

        if let Some(expr) = &node.value {
            if self.current_function == FunctionType::Initializer {
                return Err(ResolverError::ReturnInConstructor {
                    token: node.keyword,
                });
            }

            self.resolve_expr(expr)?;
        }
        Ok(())
    }

    fn visit_var(&mut self, node: &StmtVar<'a>) -> Result<(), ResolverError<'a>> {
        self.declare(&node.name)?;
        if let Some(expr) = &node.initializer {
            self.resolve_expr(expr)?;
        }
        self.define(&node.name);
        Ok(())
    }

    fn visit_while(&mut self, node: &StmtWhile<'a>) -> Result<(), ResolverError<'a>> {
        self.resolve_expr(&node.condition)?;
        self.resolve_stmt(&node.body)?;
        Ok(())
    }
}
