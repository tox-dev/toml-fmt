pub(crate) use crate::pep508::version_op::operator::Operator;
pub(crate) use crate::pep508::version_op::version::Version;

mod operator;
mod version;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionOp {
    pub op: Operator,
    pub version: Version,
}

impl VersionOp {
    pub fn new(spec: &str) -> Result<Self, String> {
        let spec = spec.trim();
        let (op, remaining) = Operator::new(spec)?;
        let version_str = remaining.trim();
        // No need to handle .*, Version::new will do it
        let version = Version::new(version_str);
        Ok(Self { op, version })
    }
}

impl std::fmt::Display for VersionOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.op, self.version)
    }
}
