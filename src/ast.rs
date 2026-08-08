#[derive(Debug, PartialEq)]
pub enum AstNode {
    Int(i64),
    Float(f64),
    String(String),
    Ident(String),
    Func(String, Vec<AstNode>),
}
