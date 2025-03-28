use std::collections::HashMap;

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

#[derive(Default)]
pub struct Resolver<'a> {
    scopes: Vec<HashMap<&'a str, bool>>,
    locals: HashMap<Expr<'a>, usize>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl<'a, 'b: 'a> Resolver<'a> {
    pub fn new() -> Self {
        Self {
            scopes: vec![],
            locals: HashMap::new(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    pub fn resolve(&mut self, stmts: &'b [Stmt<'a>]) -> Result<(), ResolverError<'a>> {
        stmts.iter().try_for_each(|stmt| self.resolve_stmt(stmt))
    }

    pub fn get_locals(self) -> HashMap<Expr<'a>, usize> {
        self.locals
    }

    fn resolve_stmt(&mut self, stmt: &'b Stmt<'a>) -> Result<(), ResolverError<'a>> {
        stmt.accept(self)
    }

    fn resolve_stmts(&mut self, stmts: &'b [Stmt<'a>]) -> Result<(), ResolverError<'a>> {
        stmts.iter().try_for_each(|stmt| self.resolve_stmt(stmt))
    }

    fn resolve_expr(&mut self, expr: &'b Expr<'a>) -> Result<(), ResolverError<'a>> {
        expr.accept(self)
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
            scope.insert(name.lexeme, false);
            Ok(())
        }
    }

    fn define(&mut self, name: &Token<'a>) {
        if self.scopes.is_empty() {
            return;
        }
        let scope = self.scopes.last_mut().unwrap();

        if scope.contains_key(name.lexeme) {
            scope.entry(name.lexeme).and_modify(|v| *v = true);
        } else {
            scope.insert(name.lexeme, true);
        }
    }

    fn resolve_local(&mut self, expr: Expr<'a>, name: Token<'a>) {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(name.lexeme) {
                self.locals.insert(expr, self.scopes.len() - 1 - i);
                return;
            }
        }
    }

    fn resolve_function(
        &mut self,
        function: &'b StmtFunction<'a>,
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

impl<'a, 'b: 'a> ExprVisitor<'a, 'b> for Resolver<'a> {
    type Output = Result<(), ResolverError<'a>>;

    fn visit_literal(&mut self, _node: &ExprLiteral) -> Self::Output {
        Ok(())
    }

    fn visit_grouping(&mut self, node: &'b ExprGrouping<'a>) -> Self::Output {
        self.resolve_expr(&node.value)?;
        Ok(())
    }

    fn visit_logical(&mut self, node: &'b ExprLogical<'a>) -> Self::Output {
        self.resolve_expr(&node.left)?;
        self.resolve_expr(&node.right)?;
        Ok(())
    }

    fn visit_set(&mut self, node: &'b ExprSet<'a>) -> Self::Output {
        self.resolve_expr(&node.value)?;
        self.resolve_expr(&node.object)?;
        Ok(())
    }

    fn visit_super(&mut self, node: &'b ExprSuper<'a>) -> Self::Output {
        if self.current_class == ClassType::None {
            return Err(ResolverError::SuperOutsideClass {
                token: node.keyword,
            });
        } else if self.current_class != ClassType::Subclass {
            return Err(ResolverError::SuperInClassWithoutSuperclass {
                token: node.keyword,
            });
        }
        self.resolve_local(Expr::Super(node.clone()), node.keyword);
        Ok(())
    }

    fn visit_this(&mut self, node: &'b ExprThis<'a>) -> Self::Output {
        if self.current_class == ClassType::None {
            return Err(ResolverError::ThisOutsideClass {
                token: node.keyword,
            });
        }

        self.resolve_local(Expr::This(node.clone()), node.keyword);
        Ok(())
    }

    fn visit_unary(&mut self, node: &'b ExprUnary<'a>) -> Self::Output {
        self.resolve_expr(&node.value)?;
        Ok(())
    }

    fn visit_binary(&mut self, node: &'b ExprBinary<'a>) -> Self::Output {
        self.resolve_expr(&node.left)?;
        self.resolve_expr(&node.right)?;
        Ok(())
    }

    fn visit_call(&mut self, node: &'b ExprCall<'a>) -> Self::Output {
        self.resolve_expr(&node.callee)?;
        node.arguments
            .iter()
            .try_for_each(|arg| self.resolve_expr(arg))
    }

    fn visit_get(&mut self, node: &'b ExprGet<'a>) -> Self::Output {
        self.resolve_expr(&node.object)?;
        Ok(())
    }

    fn visit_assign(&mut self, node: &'b ExprAssign<'a>) -> Self::Output {
        self.resolve_expr(&node.value)?;
        self.resolve_local(Expr::Assign(node.clone()), node.name);
        Ok(())
    }

    fn visit_variable(&mut self, node: &'b ExprVariable<'a>) -> Self::Output {
        if !self.scopes.is_empty() {
            let scope = self.scopes.last().unwrap();

            if let Some(var) = scope.get(node.name.lexeme) {
                if !(*var) {
                    return Err(ResolverError::VariableInOwnInitializer { token: node.name });
                }
            }
        }

        self.resolve_local(Expr::Variable(node.clone()), node.name);

        Ok(())
    }
}

impl<'a, 'b: 'a> StmtVisitor<'a, 'b> for Resolver<'a> {
    type Output = Result<(), ResolverError<'a>>;

    fn visit_block(&mut self, node: &'b StmtBlock<'a>) -> Self::Output {
        self.begin_scope();
        self.resolve_stmts(&node.statements)?;
        self.end_scope();
        Ok(())
    }

    fn visit_class(&mut self, node: &'b StmtClass<'a>) -> Self::Output {
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
            self.begin_scope();
            self.scopes.last_mut().unwrap().insert("super", true);
        }

        self.begin_scope();
        self.scopes.last_mut().unwrap().insert("this", true);

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

    fn visit_expression(&mut self, node: &'b StmtExpression<'a>) -> Self::Output {
        self.resolve_expr(&node.expr)
    }

    fn visit_function(&mut self, node: &'b StmtFunction<'a>) -> Self::Output {
        self.declare(&node.name)?;
        self.define(&node.name);

        self.resolve_function(node, FunctionType::Function)?;
        Ok(())
    }

    fn visit_if(&mut self, node: &'b StmtIf<'a>) -> Self::Output {
        self.resolve_expr(&node.condition)?;
        self.resolve_stmt(&node.then_branch)?;
        if let Some(else_branch) = &node.else_branch {
            self.resolve_stmt(else_branch)?;
        }
        Ok(())
    }

    fn visit_print(&mut self, node: &'b StmtPrint<'a>) -> Self::Output {
        self.resolve_expr(&node.expr)
    }

    fn visit_return(&mut self, node: &'b StmtReturn<'a>) -> Self::Output {
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

    fn visit_var(&mut self, node: &'b StmtVar<'a>) -> Self::Output {
        self.declare(&node.name)?;
        if let Some(expr) = &node.initializer {
            self.resolve_expr(expr)?;
        }
        self.define(&node.name);
        Ok(())
    }

    fn visit_while(&mut self, node: &'b StmtWhile<'a>) -> Self::Output {
        self.resolve_expr(&node.condition)?;
        self.resolve_stmt(&node.body)?;
        Ok(())
    }
}
