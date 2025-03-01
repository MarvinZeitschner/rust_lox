use ast_macro::Ast;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum LiteralValue {
    String(String),
    F64(f64),
}

#[derive(Ast)]
pub enum Expression {
    Literal {
        value: LiteralValue,
    },
    Binary {
        left: Box<Expr>,
        operator: String,
        right: Box<Expr>,
    },
}

struct Evaluator;

impl Visitor for Evaluator {
    type Output = f64;

    fn visit_literal(&mut self, node: &ExprLiteral) -> Self::Output {
        if let LiteralValue::F64(val) = node.value {
            return val;
        }
        0.0
    }

    fn visit_binary(&mut self, node: &ExprBinary) -> Self::Output {
        let left_val = node.left.accept(self);
        let right_val = node.right.accept(self);

        match node.operator.as_str() {
            "+" => left_val + right_val,
            "-" => left_val - right_val,
            "*" => left_val * right_val,
            "/" => left_val / right_val,
            _ => panic!("Unknown operator: {}", node.operator),
        }
    }
}

struct AstPrinter;

impl AstPrinter {
    fn parenthesize(&mut self, name: String, node: &[&Expr]) -> String {
        let expr_ac: Vec<String> = node.iter().map(|node| node.accept(self)).collect();
        format!("({} {})", name, expr_ac.join(" "))
    }
}

impl Visitor for AstPrinter {
    type Output = String;

    fn visit_literal(&mut self, node: &ExprLiteral) -> Self::Output {
        match &node.value {
            LiteralValue::String(value) => value.clone(),
            LiteralValue::F64(value) => value.to_string(),
        }
    }

    fn visit_binary(&mut self, node: &ExprBinary) -> Self::Output {
        self.parenthesize(node.operator.clone(), &[&node.left, &node.right])
    }
}

#[test]
fn expr_expansion() {
    let literal = ExprLiteral::new(LiteralValue::F64(42.0));
    if let LiteralValue::F64(lit) = literal.value {
        assert_eq!(lit, 42.0);
    }

    let expr = Expr::Literal(literal);
    if let Expr::Literal(lit) = expr {
        if let LiteralValue::F64(lit) = lit.value {
            assert_eq!(lit, 42.0);
        }
    }
}

#[test]
fn evaluator_visitor() {
    // Expr: (5 + 3) * 2 = 16
    let five = Box::new(Expr::Literal(ExprLiteral::new(LiteralValue::F64(5.0))));
    let three = Box::new(Expr::Literal(ExprLiteral::new(LiteralValue::F64(3.0))));
    let add = Box::new(Expr::Binary(ExprBinary::new(five, "+".to_string(), three)));
    let two = Box::new(Expr::Literal(ExprLiteral::new(LiteralValue::F64(2.0))));
    let mult = ExprBinary::new(add, "*".to_string(), two);
    let expr = Expr::Binary(mult);

    let mut evaluator = Evaluator;
    let result = expr.accept(&mut evaluator);

    assert_eq!(result, 16.0);
}

#[test]
fn ast_printer() {
    // Expr: (5 + 3) * 2 = 16
    let five = Box::new(Expr::Literal(ExprLiteral::new(LiteralValue::F64(5.1))));
    let three = Box::new(Expr::Literal(ExprLiteral::new(LiteralValue::F64(3.1))));
    let add = Box::new(Expr::Binary(ExprBinary::new(five, "+".to_string(), three)));
    let two = Box::new(Expr::Literal(ExprLiteral::new(LiteralValue::F64(2.1))));
    let mult = ExprBinary::new(add, "*".to_string(), two);
    let expr = Expr::Binary(mult);

    let mut printer = AstPrinter;
    let result = expr.accept(&mut printer);

    assert_eq!(result, "(* (+ 5.1 3.1) 2.1)");
}
