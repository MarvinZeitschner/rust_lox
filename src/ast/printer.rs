use super::*;

struct AstPrinter;

impl AstPrinter {
    fn parenthesize(&mut self, name: String, node: &[&Expr]) -> String {
        let expr_ac: Vec<String> = node.iter().map(|node| node.accept(self)).collect();
        format!("({} {})", name, expr_ac.join(" "))
    }
}

impl Visitor<String> for AstPrinter {
    fn visit_literal(&mut self, node: &ExprLiteral) -> String {
        match &node.value {
            LiteralValue::String(value) => value.clone(),
            LiteralValue::F64(value) => value.to_string(),
            LiteralValue::Bool(value) => value.to_string(),
            LiteralValue::Nil => "nil".to_string(),
        }
    }

    fn visit_binary(&mut self, node: &ExprBinary) -> String {
        self.parenthesize(node.operator.clone(), &[&node.left, &node.right])
    }

    fn visit_grouping(&mut self, node: &ExprGrouping) -> String {
        self.parenthesize("group".to_string(), &[&node.value])
    }

    fn visit_unary(&mut self, node: &ExprUnary) -> String {
        self.parenthesize(node.operator.clone(), &[&node.value])
    }
}
