use std::io::Write;

use crate::tokenize::Lexer;

mod token;
mod tokenize;

fn main() {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut buf = String::new();
    loop {
        buf.clear();
        print!("> ");
        stdout.flush().unwrap();
        stdin.read_line(&mut buf).unwrap();
        let mut lexer = Lexer::new(&buf);
        while let Some(token) = lexer.tokenize() {
            println!("{:?}", token);
        }
        println!("---");
    }
}
