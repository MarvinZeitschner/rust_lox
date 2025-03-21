use super::*;

pub struct AstPrinter;

impl AstPrinter {
    fn parenthesize(&mut self, name: &str, node: &[&Expr]) -> String {
        let mut expr_ac = Vec::new();
        for expr in node {
            let e = (*expr).clone();
            expr_ac.push(e.accept(self));
        }
        format!("({} {})", name, expr_ac.join(" "))
    }
}

impl<'a> ExprVisitor<'_, 'a> for AstPrinter {
    type Output = String;

    fn visit_literal(&mut self, node: &ExprLiteral) -> Self::Output {
        match &node.value {
            LiteralValue::String(value) => value.clone(),
            LiteralValue::F64(value) => value.to_string(),
            LiteralValue::Bool(value) => value.to_string(),
            LiteralValue::Nil => "nil".to_string(),
        }
    }

    fn visit_grouping(&mut self, node: &ExprGrouping) -> Self::Output {
        self.parenthesize("group", &[&node.value])
    }

    fn visit_logical(&mut self, _node: &ExprLogical) -> Self::Output {
        todo!()
    }

    fn visit_set(&mut self, _node: &ExprSet<'a>) -> Self::Output {
        todo!()
    }

    fn visit_this(&mut self, _node: &ExprThis<'a>) -> Self::Output {
        todo!()
    }

    fn visit_unary(&mut self, node: &ExprUnary) -> Self::Output {
        self.parenthesize(node.operator.lexeme, &[&node.value])
    }

    fn visit_binary(&mut self, node: &ExprBinary) -> Self::Output {
        self.parenthesize(node.operator.lexeme, &[&node.left, &node.right])
    }

    fn visit_call(&mut self, _node: &ExprCall) -> Self::Output {
        todo!()
    }

    fn visit_get(&mut self, _node: &ExprGet<'a>) -> Self::Output {
        todo!()
    }

    fn visit_assign(&mut self, _node: &ExprAssign) -> Self::Output {
        todo!()
    }

    fn visit_variable(&mut self, _node: &ExprVariable) -> Self::Output {
        todo!()
    }
}
