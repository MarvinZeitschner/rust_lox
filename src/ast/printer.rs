use super::*;

pub struct AstPrinter;

impl AstPrinter {
    pub fn print(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Literal(node) => match &node.value {
                LiteralValue::String(value) => value.clone(),
                LiteralValue::F64(value) => value.to_string(),
                LiteralValue::Bool(value) => value.to_string(),
                LiteralValue::Nil => "nil".to_string(),
            },
            Expr::Grouping(node) => self.parenthesize("group", &[&node.value]),
            Expr::Unary(node) => self.parenthesize(node.operator.lexeme, &[&node.value]),
            Expr::Binary(node) => {
                self.parenthesize(node.operator.lexeme, &[&node.left, &node.right])
            }
            Expr::Logical(_)
            | Expr::Set(_)
            | Expr::Super(_)
            | Expr::This(_)
            | Expr::Call(_)
            | Expr::Get(_)
            | Expr::Assign(_)
            | Expr::Variable(_) => todo!(),
        }
    }

    fn parenthesize(&mut self, name: &str, exprs: &[&Expr]) -> String {
        let parts: Vec<String> = exprs.iter().map(|expr| self.print(expr)).collect();
        format!("({} {})", name, parts.join(" "))
    }
}
