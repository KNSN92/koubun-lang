#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Ident(String),
    Int(i64),
    Float(f64),
    String(String),
    Symbol(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub len: usize,
}

impl Span {
    pub fn new(start: usize, len: usize) -> Self {
        Self { start, len }
    }
}
