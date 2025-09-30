macro_rules! define_operators {
    ($(($variant:ident, $string:literal)),* $(,)?) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum Operator {
            $($variant,)*
        }

        impl Operator {
            const OPERATORS: &'static [(Operator, &'static str)] = &[
                $((Operator::$variant, $string),)*
            ];

            pub fn new(s: &str) -> Result<(Self, &str), String> {
                for &(ref op, op_str) in Self::OPERATORS {
                    if let Some(stripped) = s.strip_prefix(op_str) {
                        return Ok((op.clone(), stripped));
                    }
                }
                Err(format!("Invalid operator: {}", s))
            }
        }

        impl std::fmt::Display for Operator {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let op_str = match self {
                    $(Operator::$variant => $string,)*
                };
                write!(f, "{}", op_str)
            }
        }
    };
}

define_operators! {
    (ArbitraryEqual, "==="),
    (Compatible, "~="),
    (Equal, "=="),
    (GreaterEqual, ">="),
    (GreaterThan, ">"),
    (LessEqual, "<="),
    (LessThan, "<"),
    (NotEqual, "!="),
}
