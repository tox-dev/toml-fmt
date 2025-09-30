#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkerExpr {
    And(Vec<MarkerExpr>),
    Or(Vec<MarkerExpr>),
    Comparison { left: String, op: String, right: String },
    Paren(Box<MarkerExpr>),
}

impl MarkerExpr {
    pub fn new(input: &str) -> Result<Self, String> {
        let tokens = tokenize(input)?;
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_marker()?;
        if parser.peek().is_some() {
            return Err("Unexpected trailing tokens".to_string());
        }
        Ok(expr)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token<'a> {
    Ident(&'a str),
    String(&'a str),
    Op(&'a str),
    And,
    Or,
    LParen,
    RParen,
}

fn tokenize(input: &str) -> Result<Vec<Token<'_>>, String> {
    let mut tokens = Vec::new();
    let mut i = 0;
    let chars: Vec<_> = input.chars().collect();
    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' | '\n' => i += 1,
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            '=' | '!' | '>' | '<' => {
                let mut j = i + 1;
                while j < chars.len() && "=<>!".contains(chars[j]) {
                    j += 1;
                }
                tokens.push(Token::Op(&input[i..j]));
                i = j;
            }
            '"' | '\'' => {
                let quote = chars[i];
                let mut j = i + 1;
                while j < chars.len() && chars[j] != quote {
                    j += 1;
                }
                if j == chars.len() {
                    return Err("Unclosed string literal".to_string());
                }
                tokens.push(Token::String(&input[i..=j]));
                i = j + 1;
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let mut j = i + 1;
                while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '_') {
                    j += 1;
                }
                let word = &input[i..j];
                match word {
                    "and" => tokens.push(Token::And),
                    "or" => tokens.push(Token::Or),
                    _ => tokens.push(Token::Ident(word)),
                }
                i = j;
            }
            _ => return Err(format!("Unexpected character: {}", chars[i])),
        }
    }
    Ok(tokens)
}

struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: Vec<Token<'a>>) -> Self {
        Parser { tokens, pos: 0 }
    }
    fn peek(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.pos)
    }
    fn next(&mut self) -> Option<&Token<'a>> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }
    fn parse_marker(&mut self) -> Result<MarkerExpr, String> {
        self.parse_or()
    }
    fn parse_or(&mut self) -> Result<MarkerExpr, String> {
        let mut exprs = vec![self.parse_and()?];
        while let Some(Token::Or) = self.peek() {
            self.next();
            exprs.push(self.parse_and()?);
        }
        if exprs.len() == 1 {
            Ok(exprs.remove(0))
        } else {
            Ok(MarkerExpr::Or(exprs))
        }
    }
    fn parse_and(&mut self) -> Result<MarkerExpr, String> {
        let mut exprs = vec![self.parse_atom()?];
        while let Some(Token::And) = self.peek() {
            self.next();
            exprs.push(self.parse_atom()?);
        }
        if exprs.len() == 1 {
            Ok(exprs.remove(0))
        } else {
            Ok(MarkerExpr::And(exprs))
        }
    }
    fn parse_atom(&mut self) -> Result<MarkerExpr, String> {
        match self.peek() {
            Some(Token::LParen) => {
                self.next();
                let expr = self.parse_marker()?;
                match self.next() {
                    Some(Token::RParen) => Ok(MarkerExpr::Paren(Box::new(expr))),
                    _ => Err("Expected ')'".to_string()),
                }
            }
            _ => self.parse_comparison(),
        }
    }
    fn parse_comparison(&mut self) -> Result<MarkerExpr, String> {
        let left = match self.next() {
            Some(Token::Ident(s)) => s.to_string(),
            _ => return Err("Expected identifier".to_string()),
        };
        let op = match self.next() {
            Some(Token::Op(op)) => op.to_string(),
            Some(Token::Ident("in")) => "in".to_string(),
            Some(Token::Ident("not")) => match self.next() {
                Some(Token::Ident("in")) => "not in".to_string(),
                _ => return Err("Expected 'in' after 'not'".to_string()),
            },
            _ => return Err("Expected operator".to_string()),
        };
        let right = match self.next() {
            Some(Token::String(s)) => s.to_string(),
            Some(Token::Ident(s)) => s.to_string(),
            _ => return Err("Expected string or identifier as right-hand side".to_string()),
        };
        Ok(MarkerExpr::Comparison { left, op, right })
    }
}

impl std::fmt::Display for MarkerExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarkerExpr::And(exprs) => {
                let mut first = true;
                for expr in exprs {
                    if !first {
                        write!(f, " and ")?;
                    }
                    write!(f, "{}", expr)?;
                    first = false;
                }
                Ok(())
            }
            MarkerExpr::Or(exprs) => {
                let mut first = true;
                for expr in exprs {
                    if !first {
                        write!(f, " or ")?;
                    }
                    write!(f, "{}", expr)?;
                    first = false;
                }
                Ok(())
            }
            MarkerExpr::Comparison { left, op, right } => {
                let formatted = if (right.starts_with('"') && right.ends_with('"'))
                    || (right.starts_with('\'') && right.ends_with('\''))
                {
                    let inner = &right[1..right.len() - 1];
                    format!("'{inner}'")
                } else {
                    right.to_string()
                };
                write!(f, "{left}{op}{formatted}")
            }
            MarkerExpr::Paren(expr) => {
                write!(f, "({})", expr)
            }
        }
    }
}
