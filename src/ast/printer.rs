use super::*;

pub struct AstPrinter;

impl AstPrinter {
    fn parenthesize(&mut self, name: &str, node: &[&Expr]) -> String {
        let expr_ac: Vec<String> = node.iter().map(|node| node.accept(self)).collect();
        format!("({} {})", name, expr_ac.join(" "))
    }
}

impl Visitor for AstPrinter {
    type Result = String;

    fn visit_literal(&mut self, node: &ExprLiteral) -> Self::Result {
        match &node.value {
            LiteralValue::String(value) => value.clone(),
            LiteralValue::F64(value) => value.to_string(),
            LiteralValue::Bool(value) => value.to_string(),
            LiteralValue::Nil => "nil".to_string(),
        }
    }

    fn visit_grouping(&mut self, node: &ExprGrouping) -> Self::Result {
        self.parenthesize("group", &[&node.value])
    }

    fn visit_unary(&mut self, node: &ExprUnary) -> Self::Result {
        self.parenthesize(node.operator.lexeme, &[&node.value])
    }

    fn visit_binary(&mut self, node: &ExprBinary) -> Self::Result {
        self.parenthesize(node.operator.lexeme, &[&node.left, &node.right])
    }
}
