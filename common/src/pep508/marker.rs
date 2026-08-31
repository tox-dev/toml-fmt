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
        // the parser reads a group by calling itself, and the tree it builds is read and dropped
        // the same way, so a marker nested past this says more than the machine can hold
        if tokens.iter().filter(|token| **token == Token::LParen).count() > GROUPS {
            return Err(format!("The marker groups more than {GROUPS} times"));
        }
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_marker()?;
        if parser.peek().is_some() {
            return Err("Unexpected trailing tokens".to_string());
        }
        Ok(expr)
    }
}

/// How many groups a marker may open. PEP 508 sets no limit, and a marker written by hand carries
/// a handful; the bound is what a thread's stack holds for reading, writing and dropping the tree.
const GROUPS: usize = 256;

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
    // the positions here are byte offsets, since that is what a slice of the input is written in
    let mut tokens = Vec::new();
    let mut at = 0;
    while let Some(held) = input[at..].chars().next() {
        let width = held.len_utf8();
        match held {
            // a marker is written on one line, so a break in it is not spacing
            ' ' | '\t' => at += width,
            '(' => {
                tokens.push(Token::LParen);
                at += width;
            }
            ')' => {
                tokens.push(Token::RParen);
                at += width;
            }
            '=' | '!' | '>' | '<' | '~' => {
                let end = run_end(input, at, |held| "=<>!~".contains(held));
                tokens.push(Token::Op(&input[at..end]));
                at = end;
            }
            '"' | '\'' => {
                let Some(close) = input[at + width..].find(held) else {
                    return Err("Unclosed string literal".to_string());
                };
                let end = at + width + close + width;
                tokens.push(Token::String(&input[at..end]));
                at = end;
            }
            held if held.is_ascii_alphabetic() || held == '_' => {
                let end = run_end(input, at, |held| held.is_ascii_alphanumeric() || held == '_');
                let word = &input[at..end];
                match word {
                    "and" => tokens.push(Token::And),
                    "or" => tokens.push(Token::Or),
                    _ => tokens.push(Token::Ident(word)),
                }
                at = end;
            }
            other => return Err(format!("Unexpected character: {other}")),
        }
    }
    Ok(tokens)
}

/// Where the run of characters `holds` accepts, starting at `from`, ends.
fn run_end(input: &str, from: usize, holds: impl Fn(char) -> bool) -> usize {
    input[from..]
        .find(|held: char| !holds(held))
        .map_or(input.len(), |width| from + width)
}

/// What a marker compares with, from the
/// [dependency specifier grammar](https://packaging.python.org/en/latest/specifications/dependency-specifiers/).
const MARKER_OPERATORS: &[&str] = &["==", "!=", "<", "<=", ">", ">=", "~=", "==="];

/// The environment a marker may name. Anything else on that side is a value, which is quoted.
const MARKER_VARIABLES: &[&str] = &[
    "os_name",
    "sys_platform",
    "platform_machine",
    "platform_python_implementation",
    "platform_release",
    "platform_system",
    "platform_version",
    "python_version",
    "python_full_version",
    "implementation_name",
    "implementation_version",
    "extra",
    "dependency_groups",
];

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
        let left = self.parse_operand()?;
        let op = match self.next() {
            // PEP 508 names the operators a marker compares with, and nothing else is one
            Some(Token::Op(op)) if MARKER_OPERATORS.contains(op) => (*op).to_string(),
            Some(Token::Op(op)) => return Err(format!("`{op}` is no marker operator")),
            Some(Token::Ident("in")) => "in".to_string(),
            Some(Token::Ident("not")) => match self.next() {
                Some(Token::Ident("in")) => "not in".to_string(),
                _ => return Err("Expected 'in' after 'not'".to_string()),
            },
            _ => return Err("Expected operator".to_string()),
        };
        let right = self.parse_operand()?;
        Ok(MarkerExpr::Comparison { left, op, right })
    }

    /// One side of a comparison: a quoted value, or one of the variables PEP 508 names.
    fn parse_operand(&mut self) -> Result<String, String> {
        match self.next() {
            Some(Token::String(held)) => Ok((*held).to_string()),
            Some(Token::Ident(held)) if MARKER_VARIABLES.contains(held) => Ok((*held).to_string()),
            Some(Token::Ident(held)) => Err(format!("`{held}` is no marker variable, and a value is quoted")),
            _ => Err("Expected a quoted value or a marker variable".to_string()),
        }
    }
}

/// The expressions written out with `between` holding them together.
fn joined(exprs: &[MarkerExpr], between: &str) -> String {
    exprs
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(between)
}

impl std::fmt::Display for MarkerExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarkerExpr::And(exprs) => f.write_str(&joined(exprs, " and ")),
            MarkerExpr::Or(exprs) => f.write_str(&joined(exprs, " or ")),
            MarkerExpr::Comparison { left, op, right } => {
                let formatted = if (right.starts_with('"') && right.ends_with('"'))
                    || (right.starts_with('\'') && right.ends_with('\''))
                {
                    // the quote a value holds cannot be the one that closes it, and PEP 508 has no
                    // escape for either, so the other one writes it
                    let inner = &right[1..right.len() - 1];
                    if inner.contains('\'') {
                        format!("\"{inner}\"")
                    } else {
                        format!("'{inner}'")
                    }
                } else {
                    right.to_string()
                };
                if op == "in" || op == "not in" {
                    write!(f, "{left} {op} {formatted}")
                } else {
                    write!(f, "{left}{op}{formatted}")
                }
            }
            MarkerExpr::Paren(expr) => {
                write!(f, "({})", expr)
            }
        }
    }
}
