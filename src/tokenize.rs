use std::{collections::HashMap, iter::Peekable, str::Chars, sync::LazyLock};

use crate::token::Token;

pub struct Lexer<'a> {
    source_iter: Peekable<Chars<'a>>,
    counter: usize,
}

const SYMBOLS: &[char] = &[
    '(', ')', '{', '}', '[', ']', ';', ',', '.', '+', '-', '*', '/', '%', '=', '<', '>', '!', '&',
    '|', '^', '~', '?', ':', '$', '@', '#',
];

static GENERAL_ESCAPE_SEQUENCES: LazyLock<HashMap<char, char>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert('n', '\n');
    map.insert('r', '\r');
    map.insert('t', '\t');
    map.insert('\\', '\\');
    map
});

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let source_iter = source.chars().peekable();
        Lexer {
            source_iter,
            counter: 0,
        }
    }

    pub fn tokenize(&mut self) -> Option<Token> {
        self.skip_whitespace();
        if let Some(token) = self.parse_ident() {
            return Some(token);
        }
        if let Some(token) = self.parse_number() {
            return Some(token);
        }
        if let Some(token) = self.parse_string() {
            return Some(token);
        }
        if let Some(token) = self.parse_symbol() {
            return Some(token);
        }
        None
    }

    fn forward(&mut self) -> Option<char> {
        self.counter += 1;
        self.source_iter.next();
        self.peek()
    }

    fn peek(&mut self) -> Option<char> {
        self.source_iter.peek().copied()
    }

    fn skip_whitespace(&mut self) {
        match self.peek() {
            Some(c) if !c.is_whitespace() => return,
            None => return,
            _ => {}
        }
        while let Some(c) = self.forward()
            && c.is_whitespace()
        {}
    }

    fn parse_ident(&mut self) -> Option<Token> {
        let mut ident = String::new();
        if let Some(c) = self.peek()
            && (c.is_ascii_alphabetic() || c == '_')
        {
            ident.push(c);
        } else {
            return None;
        }
        while let Some(c) = self.forward()
            && (c.is_ascii_alphanumeric() || c == '_')
        {
            ident.push(c);
        }
        Some(Token::Ident(ident))
    }

    fn parse_number(&mut self) -> Option<Token> {
        let mut number = String::new();
        if let Some(c) = self.peek()
            && c.is_numeric()
        {
            number.push(c);
        } else {
            return None;
        }
        while let Some(c) = self.forward()
            && c.is_numeric()
        {
            number.push(c);
        }
        if self.peek() == Some('.') {
            number.push('.');
            while let Some(c) = self.forward()
                && c.is_numeric()
            {
                number.push(c);
            }
        }
        if self.peek() == Some('e') || self.peek() == Some('E') {
            number.push(self.peek().unwrap());
            self.forward();
            match self.peek() {
                Some(c) if c == '+' || c == '-' => {
                    number.push(c);
                    self.forward();
                }
                None => return None,
                _ => {}
            }
            while let Some(c) = self.peek()
                && c.is_numeric()
            {
                number.push(c);
                self.forward();
            }
        }
        if number.chars().all(|c| c.is_digit(10)) {
            Some(Token::Int(number.parse::<i64>().unwrap()))
        } else {
            Some(Token::Float(number.parse::<f64>().unwrap()))
        }
    }

    fn parse_string(&mut self) -> Option<Token> {
        if self.peek() != Some('"') && self.peek() != Some('\'') {
            return None;
        }
        let quote = self.peek().unwrap();
        let mut string = String::new();
        while let Some(c) = self.forward()
            && c != quote
        {
            if c == '\\' {
                if let Some(escaped) = self.forward() {
                    match GENERAL_ESCAPE_SEQUENCES.get(&escaped) {
                        Some(&general_escaped) => string.push(general_escaped),
                        None if quote == '"' && escaped == '"' => string.push('"'),
                        None if quote == '\'' && escaped == '\'' => string.push('\''),
                        None => string.push(escaped), // unrecognized escape sequence, treat as literal
                    }
                } else {
                    // unexpected eof
                }
            } else {
                string.push(c);
            }
        }
        self.forward();
        Some(Token::String(string))
    }

    fn parse_symbol(&mut self) -> Option<Token> {
        if let Some(c) = self.peek() {
            if SYMBOLS.contains(&c) {
                self.forward();
                return Some(Token::Symbol(c.to_string()));
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use crate::{token::Token, tokenize::Lexer};

    #[test]
    fn test_ident_hello() {
        let source = "hello";
        let mut lexer = Lexer::new(source);
        let token = lexer.tokenize().unwrap();
        assert_eq!(token, Token::Ident("hello".to_string()));
    }

    #[test]
    fn test_ident_first_underscore() {
        let source = "_hello";
        let mut lexer = Lexer::new(source);
        let token = lexer.tokenize().unwrap();
        assert_eq!(token, Token::Ident("_hello".to_string()));
    }

    #[test]
    fn test_ident_after_second_number() {
        let source = "hello123";
        let mut lexer = Lexer::new(source);
        let token = lexer.tokenize().unwrap();
        assert_eq!(token, Token::Ident("hello123".to_string()));
    }

    #[test]
    fn test_ident_before_whitespace() {
        let source = "    hello";
        let mut lexer = Lexer::new(source);
        let token = lexer.tokenize().unwrap();
        assert_eq!(token, Token::Ident("hello".to_string()));
    }

    #[test]
    fn test_number_int_zero() {
        let source = "0";
        let mut lexer = Lexer::new(source);
        let token = lexer.tokenize().unwrap();
        assert_eq!(token, Token::Int(0));
    }

    #[test]
    fn test_number_int_123() {
        let source = "123";
        let mut lexer = Lexer::new(source);
        let token = lexer.tokenize().unwrap();
        assert_eq!(token, Token::Int(123));
    }

    #[test]
    fn test_number_int_leading_zero() {
        let source = "0123";
        let mut lexer = Lexer::new(source);
        let token = lexer.tokenize().unwrap();
        assert_eq!(token, Token::Int(123));
    }

    #[test]
    fn test_number_float_0() {
        let source = "0.0";
        let mut lexer = Lexer::new(source);
        let token = lexer.tokenize().unwrap();
        assert_eq!(token, Token::Float(0.0));
    }

    #[test]
    fn test_number_float_123_456() {
        let source = "123.456";
        let mut lexer = Lexer::new(source);
        let token = lexer.tokenize().unwrap();
        assert_eq!(token, Token::Float(123.456));
    }

    #[test]
    fn test_number_float_no_decimal() {
        let source = "123.";
        let mut lexer = Lexer::new(source);
        let token = lexer.tokenize().unwrap();
        assert_eq!(token, Token::Float(123.0));
    }

    #[test]
    fn test_number_float_leading_zero() {
        let source = "0.123";
        let mut lexer = Lexer::new(source);
        let token = lexer.tokenize().unwrap();
        assert_eq!(token, Token::Float(0.123));
    }

    #[test]
    fn test_number_float_trailing_zero() {
        let source = "123.4560";
        let mut lexer = Lexer::new(source);
        let token = lexer.tokenize().unwrap();
        assert_eq!(token, Token::Float(123.4560));
    }

    #[test]
    fn test_number_float_exponent() {
        let source = "123.456e-10";
        let mut lexer = Lexer::new(source);
        let token = lexer.tokenize().unwrap();
        assert_eq!(token, Token::Float(123.456e-10));
    }

    #[test]
    fn test_number_float_exponent_plus() {
        let source = "123.456e+2";
        let mut lexer = Lexer::new(source);
        let token = lexer.tokenize().unwrap();
        assert_eq!(token, Token::Float(12345.6));
    }

    #[test]
    fn test_number_float_exponent_no_sign() {
        let source = "123.456e20";
        let mut lexer = Lexer::new(source);
        let token = lexer.tokenize().unwrap();
        assert_eq!(token, Token::Float(123.456e20));
    }

    #[test]
    fn test_number_float_exponent_no_decimal() {
        let source = "123e-10";
        let mut lexer = Lexer::new(source);
        let token = lexer.tokenize().unwrap();
        assert_eq!(token, Token::Float(123.0e-10));
    }

    #[test]
    fn test_number_float_exponent_numberless() {
        let source = "123.456e";
        let mut lexer = Lexer::new(source);
        let token = lexer.tokenize();
        assert_eq!(token, None);
    }
}
