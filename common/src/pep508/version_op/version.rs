use std::sync::LazyLock;

use regex::Regex;

/// Whether `raw` is a valid [PEP 440](https://peps.python.org/pep-0440/) version.
pub fn is_valid_version(raw: &str) -> bool {
    PEP440.is_match(raw.trim())
}

static PEP440: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?xi)
        ^
        v?
        (?:(?P<epoch>[0-9]+)!)?
        (?P<release>[0-9]+(?:\.[0-9]+)*)
        (?:[-_.]?(?P<pre_l>alpha|a|beta|b|preview|pre|c|rc)[-_.]?(?P<pre_n>[0-9]+)?)?
        (?:-(?P<post_n1>[0-9]+)|[-_.]?(?P<post_l>post|rev|r)[-_.]?(?P<post_n2>[0-9]+)?)?
        (?P<dev>[-_.]?dev[-_.]?(?P<dev_n>[0-9]+)?)?
        (?:\+(?P<local>[a-z0-9]+(?:[-_.][a-z0-9]+)*))?
        $",
    )
    .unwrap()
});

/// A number a version writes, held as the digits the file wrote rather than as a machine integer:
/// PEP 440 puts no limit on how many of them a release names.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Number(String);

impl Number {
    /// Read the digits, with the leading zeros PEP 440 does not count dropped.
    fn read(digits: &str) -> Self {
        let held = digits.trim_start_matches('0');
        Self(if held.is_empty() {
            String::from("0")
        } else {
            held.to_owned()
        })
    }

    /// The value as a machine integer, saturating where it names more than one can hold. A caller
    /// comparing against a small number reads the same answer either way.
    #[must_use]
    pub fn saturating(&self) -> u64 {
        self.0.parse().unwrap_or(u64::MAX)
    }

    /// Whether the number is zero, which is what a trailing `.0` says.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0 == "0"
    }

    /// Zero, which is what a release leaves unwritten.
    #[must_use]
    pub fn zero() -> Self {
        Self(String::from("0"))
    }

    /// The number the file wrote, where what it wrote is digits counted the way PEP 440 counts
    /// them: a leading zero names no number of its own.
    #[must_use]
    pub fn written(digits: &str) -> Option<Self> {
        if digits.is_empty() || !digits.bytes().all(|held| held.is_ascii_digit()) {
            return None;
        }
        let held = Self::read(digits);
        (held.0 == digits).then_some(held)
    }

    /// The next number up.
    #[must_use]
    pub fn succ(&self) -> Self {
        let mut digits: Vec<u8> = self.0.bytes().rev().collect();
        let mut carry = 1;
        for digit in &mut digits {
            let held = *digit - b'0' + carry;
            *digit = b'0' + held % 10;
            carry = held / 10;
        }
        if carry == 1 {
            digits.push(b'1');
        }
        digits.reverse();
        Self(String::from_utf8(digits).expect("digits are ASCII"))
    }

    /// The next number down, or `None` where there is none below it.
    #[must_use]
    pub fn pred(&self) -> Option<Self> {
        if self.is_zero() {
            return None;
        }
        let mut digits: Vec<u8> = self.0.bytes().rev().collect();
        let mut borrow = 1;
        for digit in &mut digits {
            let held = *digit - b'0' + 10 - borrow;
            *digit = b'0' + held % 10;
            borrow = 1 - held / 10;
        }
        digits.reverse();
        Some(Self::read(&String::from_utf8(digits).expect("digits are ASCII")))
    }
}

impl Ord for Number {
    /// Longer digits name a larger number, and the same count of them compares digit by digit.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.len().cmp(&other.0.len()).then_with(|| self.0.cmp(&other.0))
    }
}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub epoch: Option<Number>,
    pub release: Vec<Number>,
    pub pre: Option<(String, Option<Number>)>,
    pub post: Option<(Option<String>, Option<Number>)>,
    pub dev: Option<(String, Option<Number>)>,
    pub local: Option<String>,
    pub has_wildcard: bool,
}

impl Version {
    /// Read a version the way PEP 440 spells one.
    ///
    /// # Errors
    ///
    /// Returns why the text is not a version this can write back out unchanged: text PEP 440 does
    /// not read as a version, or a release number too large to hold.
    pub fn new(raw: &str) -> Result<Self, String> {
        let mut input = raw.trim();
        let has_wildcard = input.ends_with(".*");
        if has_wildcard {
            input = &input[..input.len() - 2];
        }
        let found = PEP440
            .captures(input)
            .ok_or_else(|| format!("Invalid version: {input}"))?;
        let number = |name: &str| found.name(name).map(|held| Number::read(held.as_str()));
        let release = found
            .name("release")
            .expect("the release is not optional")
            .as_str()
            .split('.')
            .map(Number::read)
            .collect();
        // an implicit post release is written `1.0-1`, which names the number without the label
        let post = match (found.name("post_n1"), found.name("post_l")) {
            (Some(held), _) => Some((None, Some(Number::read(held.as_str())))),
            (None, Some(_)) => Some((Some(String::from("post")), number("post_n2"))),
            (None, None) => None,
        };
        Ok(Self {
            epoch: number("epoch"),
            release,
            pre: found
                .name("pre_l")
                .map(|held| (held.as_str().to_ascii_lowercase(), number("pre_n"))),
            post,
            dev: found.name("dev").map(|_| (String::from("dev"), number("dev_n"))),
            local: found.name("local").map(|held| held.as_str().to_ascii_lowercase()),
            has_wildcard,
        })
    }
}

impl Version {
    /// The version as PEP 440 spells one: each label in its canonical form, and a segment the file
    /// left without a number written with the zero the spec reads it as.
    fn spelled(&self) -> String {
        let mut written = String::new();
        if let Some(epoch) = &self.epoch {
            written.push_str(&epoch.0);
            written.push('!');
        }
        for (at, part) in self.release.iter().enumerate() {
            if at > 0 {
                written.push('.');
            }
            written.push_str(&part.0);
        }
        if let Some((pre_l, pre_n)) = &self.pre {
            written.push_str(match pre_l.as_str() {
                "alpha" | "a" => "a",
                "beta" | "b" => "b",
                "rc" | "c" | "pre" | "preview" => "rc",
                _ => pre_l,
            });
            written.push_str(numbered(pre_n));
        }
        if let Some((_, post_n)) = &self.post {
            written.push_str(".post");
            written.push_str(numbered(post_n));
        }
        if let Some((_, dev_n)) = &self.dev {
            written.push_str(".dev");
            written.push_str(numbered(dev_n));
        }
        if let Some(local) = &self.local {
            written.push('+');
            written.push_str(&local.replace(['-', '_'], "."));
        }
        if self.has_wildcard {
            written.push_str(".*");
        }
        written
    }
}

/// The digits a segment holds, or the zero PEP 440 reads where the file wrote none.
fn numbered(held: &Option<Number>) -> &str {
    held.as_ref().map_or("0", |number| &number.0)
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.spelled())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_unknown_pre_release_label() {
        let version = Version {
            epoch: None,
            release: ["1", "2", "3"].map(Number::read).to_vec(),
            pre: Some((String::from("unknown"), Some(Number::read("1")))),
            post: None,
            dev: None,
            local: None,
            has_wildcard: false,
        };
        assert_eq!(version.to_string(), "1.2.3unknown1");
    }

    #[test]
    fn test_display_every_part_a_version_can_hold() {
        let version = Version {
            epoch: Some(Number::read("2")),
            release: ["1", "2", "3"].map(Number::read).to_vec(),
            pre: Some((String::from("xyz"), None)),
            post: Some((Some(String::from("post")), Some(Number::read("4")))),
            dev: Some((String::from("dev"), Some(Number::read("5")))),
            local: Some(String::from("a-b_c")),
            has_wildcard: true,
        };
        assert_eq!(version.to_string(), "2!1.2.3xyz0.post4.dev5+a.b.c.*");
    }

    #[test]
    fn test_display_the_parts_that_carry_no_number() {
        let version = Version {
            epoch: None,
            release: vec![Number::read("1")],
            pre: None,
            post: Some((Some(String::from("post")), None)),
            dev: Some((String::from("dev"), None)),
            local: None,
            has_wildcard: false,
        };
        assert_eq!(version.to_string(), "1.post0.dev0");
    }
}
