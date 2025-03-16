use std::collections::HashMap;

use crate::{ast::*, lex::Token};

use super::{
    error::{ResolverError, RuntimeError},
    Interpreter,
};

pub struct Resolver<'a> {
    interpreter: Interpreter<'a>,
    scopes: Vec<HashMap<&'a str, bool>>,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: Interpreter<'a>) -> Self {
        Self {
            interpreter,
            scopes: vec![],
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt<'a>) {
        stmt.accept(self);
    }

    fn resolve_stmts(&mut self, stmts: &[Stmt<'a>]) {
        stmts.iter().for_each(|stmt| self.resolve_stmt(stmt));
    }

    fn resolve_expr(&mut self, expr: &Expr<'a>) {
        expr.accept(self);
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token<'a>) {
        let Some(scope) = self.scopes.last_mut() else {
            return;
        };

        if scope.contains_key(name.lexeme) {
            scope.entry(name.lexeme).and_modify(|v| *v = false);
        } else {
            scope.insert(name.lexeme, false);
        }
    }

    fn define(&mut self, name: &Token<'a>) {
        let Some(scope) = self.scopes.last_mut() else {
            return;
        };

        if scope.contains_key(name.lexeme) {
            scope.entry(name.lexeme).and_modify(|v| *v = true);
        }
    }

    fn resolve_local(&mut self, expr: &Expr<'a>, name: Token<'a>) {
        self.scopes.iter().rev().enumerate().for_each(|(i, scope)| {
            if scope.contains_key(name.lexeme) {
                self.interpreter.resolve(expr, i);
                return;
            }
        });
    }

    fn resolve_function(&mut self, function: &StmtFunction<'a>) {
        self.begin_scope();
        function.params.iter().for_each(|param| {
            self.declare(param);
            self.define(param);
        });
        self.resolve_stmts(&function.body);
        self.end_scope();
    }
}

impl<'a, 'b> ExprVisitor<'a, 'b> for Resolver<'a> {
    type Output = Result<(), ResolverError<'a>>;

    fn visit_literal(&mut self, _node: &ExprLiteral) -> Self::Output {
        Ok(())
    }

    fn visit_grouping(&mut self, node: &'b ExprGrouping<'a>) -> Self::Output {
        self.resolve_expr(&node.value);
        Ok(())
    }

    fn visit_logical(&mut self, node: &'b ExprLogical<'a>) -> Self::Output {
        self.resolve_expr(&node.left);
        self.resolve_expr(&node.right);
        Ok(())
    }

    fn visit_unary(&mut self, node: &'b ExprUnary<'a>) -> Self::Output {
        self.resolve_expr(&node.value);
        Ok(())
    }

    fn visit_binary(&mut self, node: &'b ExprBinary<'a>) -> Self::Output {
        self.resolve_expr(&node.left);
        self.resolve_expr(&node.right);
        Ok(())
    }

    fn visit_call(&mut self, node: &'b ExprCall<'a>) -> Self::Output {
        self.resolve_expr(&node.callee);
        node.arguments.iter().for_each(|arg| self.resolve_expr(arg));
        Ok(())
    }

    fn visit_assign(&mut self, node: &'b ExprAssign<'a>) -> Self::Output {
        self.resolve_expr(&node.value);
        self.resolve_local(&node.value, node.name);
        Ok(())
    }

    fn visit_variable(&mut self, node: &'b ExprVariable<'a>) -> Self::Output {
        let Some(scope) = self.scopes.last_mut() else {
            return Err(ResolverError::VariableInOwnInitializer { token: node.name });
        };

        let Some(var) = scope.get(node.name.lexeme) else {
            return Err(ResolverError::VariableInOwnInitializer { token: node.name });
        };

        if !(*var) {
            return Err(ResolverError::VariableInOwnInitializer { token: node.name });
        }

        // TODO: clone
        self.resolve_local(&Expr::Variable(node.clone()), node.name);

        Ok(())
    }
}

impl<'a, 'b> StmtVisitor<'a, 'b> for Resolver<'a> {
    type Output = ();

    fn visit_block(&mut self, node: &'b StmtBlock<'a>) -> Self::Output {
        self.begin_scope();
        self.resolve_stmts(&node.statements);
        self.end_scope();
    }

    fn visit_expression(&mut self, node: &'b StmtExpression<'a>) -> Self::Output {
        self.resolve_expr(&node.expr);
    }

    fn visit_function(&mut self, node: &'b StmtFunction<'a>) -> Self::Output {
        self.declare(&node.name);
        self.define(&node.name);

        self.resolve_function(node);
    }

    fn visit_if(&mut self, node: &'b StmtIf<'a>) -> Self::Output {
        self.resolve_expr(&node.condition);
        self.resolve_stmt(&node.then_branch);
        if let Some(else_branch) = &node.else_branch {
            self.resolve_stmt(else_branch);
        }
    }

    fn visit_print(&mut self, node: &'b StmtPrint<'a>) -> Self::Output {
        self.resolve_expr(&node.expr);
    }

    fn visit_return(&mut self, node: &'b StmtReturn<'a>) -> Self::Output {
        if let Some(expr) = &node.value {
            self.resolve_expr(expr);
        }
    }

    fn visit_var(&mut self, node: &'b StmtVar<'a>) -> Self::Output {
        self.declare(&node.name);
        if let Some(expr) = &node.initializer {
            self.resolve_expr(expr);
        }
        self.define(&node.name);
    }

    fn visit_while(&mut self, node: &'b StmtWhile<'a>) -> Self::Output {
        self.resolve_expr(&node.condition);
        self.resolve_stmt(&node.body);
    }
}
