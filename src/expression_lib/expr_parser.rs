use crate::Float;

use num_complex::Complex32;

pub static ALPHABET: &'static [u8] = &[
    b'+', b'-', b'*', b'/', b'^', b's', b'c', b'l', b'r', b' ', b'x', b'0', b'1', b'2', b'3', b'4',
    b'5', b'6', b'7', b'8', b'9', b'.',
];

struct Tokenizer<'a>(&'a [u8]);

impl<'a> Tokenizer<'a> {
    fn new(expr: &'a [u8]) -> Self {
        Tokenizer(expr)
    }

    fn is_num(&self, i: usize) -> bool {
        if i >= self.0.len() {
            return false;
        }

        let c = self.0[i];
        return b'0' <= c && c <= b'9' || c == b'.';
    }

    fn is_char(&self, i: usize, c: u8) -> bool {
        if i >= self.0.len() {
            return false;
        }
        return self.0[i] == c;
    }

    fn advance(&mut self, by: usize) -> &'a [u8] {
        let ret = &self.0[..by];
        self.0 = &self.0[by..];
        return ret;
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        while !self.0.is_empty() && self.0[0].is_ascii_whitespace() {
            self.advance(1);
        }

        if self.0.is_empty() {
            return None;
        }
        if !(self.is_num(0) || self.is_char(0, b'-') && self.is_num(1)) {
            let ret = self.advance(1);
            return Some(ret);
        }

        let mut i = if self.is_char(0, b'-') { 1 } else { 0 };

        while self.is_num(i) {
            i += 1;
        }

        let ret = self.advance(i);
        return Some(ret);
    }
}

trait Tokenize {
    fn tokenize(&self) -> Tokenizer;
}

impl<'a> Tokenize for &'a [u8] {
    fn tokenize(&self) -> Tokenizer {
        Tokenizer::new(self)
    }
}

fn parse_num<T>(expr: T) -> Option<Float>
where
    T: AsRef<[u8]>,
{
    let mut has_dot = false;
    let buf: Vec<_> = expr
        .as_ref()
        .into_iter()
        .cloned()
        .filter(|&c| {
            if c == b'.' {
                if !has_dot {
                    has_dot = true;
                    true
                } else {
                    false
                }
            } else {
                true
            }
        })
        .collect();

    if buf == b"." || buf == b"-." {
        return Some(0.0);
    }

    return lexical::parse(buf).ok();
}

pub fn evaluate_expression<T>(expr: T, x: Float) -> Float
where
    T: AsRef<[u8]>,
{
    let mut stack = Vec::<Complex32>::new();

    for token in expr.as_ref().tokenize() {
        // dbg!(&stack, String::from_utf8_lossy(token));
        match token {
            b"+" | b"-" => {
                let b = stack.pop().unwrap_or_default();
                let a = stack.pop().unwrap_or_default();
                stack.push(if token == b"+" { a + b } else { a - b });
            }
            b"*" | b"/" => {
                let b = stack.pop().unwrap_or(Complex32::from(1.));
                let a = stack.pop().unwrap_or(Complex32::from(1.));
                stack.push(if token == b"*" { a * b } else { a / b });
            }
            b"^" => {
                let b = stack.pop().unwrap_or(Complex32::from(1.));
                let a = stack.pop().unwrap_or(Complex32::from(1.));
                stack.push(a.powf(b.re));
            }
            b"s" | b"c" => {
                let a = stack.pop().unwrap_or_default();
                stack.push(if token == b"s" { a.sin() } else { a.cos() });
            }
            b"l" => {
                let a = stack.pop().unwrap_or(Complex32::from(1.));
                stack.push(a.ln());
            }
            b"r" => {
                let a = stack.pop().unwrap_or(Complex32::from(1.));
                stack.push(a.sqrt());
            }
            b"x" => {
                stack.push(Complex32::from(x));
            }
            _ => {
                if let Some(num) = parse_num(token) {
                    stack.push(Complex32::from(num));
                } else {
                    panic!("unknown token {:?}", String::from_utf8_lossy(token))
                }
            }
        }
    }

    return stack
        .into_iter()
        .rfind(|n| !n.is_nan())
        .unwrap_or_default()
        .re;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ops_simple() {
        for &op in &ALPHABET[..9] {
            let expr = format!("1 1 {}", op as char);
            let evald = eval(expr, 0.0);
            assert!(!evald.is_nan());
        }
    }

    #[test]
    fn test_ops_negative() {
        for &op in &ALPHABET[..9] {
            let expr = format!("-11 -1 {}", op as char);
            let evald = eval(expr, 0.0);
            assert!(!evald.is_nan());
        }
    }

    #[test]
    fn test_llc() {
        let expr = b"llc";
        let evald = eval(expr, 0.0);
        assert!(!evald.is_nan());
    }

    #[test]
    fn test_dashdot() {
        let expr = b"-.";
        let evald = eval(expr, 0.0);
        assert!(!evald.is_nan());
    }
}
