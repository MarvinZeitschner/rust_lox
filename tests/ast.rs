use ast_macro::Ast;

#[derive(Ast, Debug)]
pub enum Expr {
    Literal {
        value: String,
    },
    Binary {
        left: Box<Expr>,
        right: Box<Expr>,
        operator: String,
    },
}

#[test]
fn test_ast_macro() {
    let literal = ExprLiteral::new("42".to_string());
    assert_eq!(literal.value, "42");

    let expr = ExprAst::Literal(literal);
    if let ExprAst::Literal(lit) = expr {
        assert_eq!(lit.value, "42");
    }
}
