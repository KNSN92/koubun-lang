#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Ident(String),
    Int(i64),
    Float(f64),
    String(String),
    Symbol(String),
}
