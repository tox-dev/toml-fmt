pub use crate::pep508::version_op::operator::Operator;
pub use crate::pep508::version_op::version::is_valid_version;
pub use crate::pep508::version_op::version::{Number, Version};

mod operator;
mod version;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionOp {
    pub op: Operator,
    version: Option<Version>,
    literal: String,
}

impl VersionOp {
    /// Read one clause of a specifier set.
    ///
    /// # Errors
    ///
    /// Returns why the text is not a clause: an operator this does not know, an operand that is not
    /// a version, or a pair PEP 440 does not put together.
    pub fn new(spec: &str) -> Result<Self, String> {
        let (op, remaining) = Operator::new(spec.trim())?;
        let literal = remaining.trim().to_owned();
        if literal.is_empty() {
            return Err(format!("The clause names no version: '{spec}'"));
        }
        // `===` compares the text it is given, which is anything the version token of a dependency
        // may hold rather than a version this reads
        if op == Operator::ArbitraryEqual {
            if !literal
                .chars()
                .all(|held| held.is_ascii_alphanumeric() || "-_.*+!".contains(held))
            {
                return Err(format!(
                    "The version is written outside its own characters: '{literal}'"
                ));
            }
            return Ok(Self {
                version: Version::new(&literal).ok(),
                op,
                literal,
            });
        }
        // Version::new parses the `.*` wildcard itself.
        let version = Version::new(&literal)?;
        holds_together(&op, &version)?;
        Ok(Self {
            op,
            version: Some(version),
            literal,
        })
    }

    /// The version the clause compares against, or `None` where `===` was given text that is not
    /// one.
    #[must_use]
    pub fn version(&self) -> Option<&Version> {
        self.version.as_ref()
    }

    /// The text the clause was given, which is what `===` compares against.
    #[must_use]
    pub fn literal(&self) -> &str {
        &self.literal
    }

    pub(crate) fn version_mut(&mut self) -> Option<&mut Version> {
        self.version.as_mut()
    }
}

/// Whether PEP 440 puts this operator and this operand together.
fn holds_together(op: &Operator, version: &Version) -> Result<(), String> {
    if version.has_wildcard {
        if !matches!(op, Operator::Equal | Operator::NotEqual) {
            return Err(format!("`{op}` names no version ending in `.*`"));
        }
        // a wildcard stands for the numbers a release goes on to name, and there are none to name
        // after what these say
        if version.pre.is_some() || version.post.is_some() || version.dev.is_some() || version.local.is_some() {
            return Err(String::from("`.*` follows the release numbers alone"));
        }
    }
    // a local version says which build of a release it is, which only the operators that compare
    // one release with another read
    if version.local.is_some() && !matches!(op, Operator::Equal | Operator::NotEqual) {
        return Err(format!("`{op}` names no local version"));
    }
    // a compatible release names the version it holds the last number of open
    if *op == Operator::Compatible && version.release.len() < 2 {
        return Err(format!("`{op}` names a version of at least two numbers"));
    }
    Ok(())
}

impl std::fmt::Display for VersionOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.version {
            // `===` compares the text it was given, so that is what is written back
            Some(version) if self.op != Operator::ArbitraryEqual => write!(f, "{}{}", self.op, version),
            _ => write!(f, "{}{}", self.op, self.literal),
        }
    }
}
