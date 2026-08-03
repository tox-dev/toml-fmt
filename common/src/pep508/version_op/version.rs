use std::sync::LazyLock;

use regex::{Captures, Regex};

/// Canonical [PEP 440](https://peps.python.org/pep-0440/) form of `raw`, or `None` when `raw` is not a valid version.
pub fn normalize_version(raw: &str) -> Option<String> {
    Version::parse(raw).map(|version| version.to_string())
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

/// Outer `None` rejects the whole version when the captured digits overflow, inner `None` means the group was absent.
fn optional_number(caps: &Captures, name: &str) -> Option<Option<u64>> {
    caps.name(name)
        .map_or(Some(None), |number| number.as_str().parse().ok().map(Some))
}

fn normalize_local(raw: &str) -> String {
    let lowered = raw.to_ascii_lowercase();
    lowered
        .split(['-', '_', '.'])
        .map(|part| {
            if part.chars().all(|c| c.is_ascii_digit()) {
                match part.trim_start_matches('0') {
                    "" => "0",
                    trimmed => trimmed,
                }
            } else {
                part
            }
        })
        .collect::<Vec<&str>>()
        .join(".")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub epoch: Option<u64>,
    pub release: Vec<u64>,
    pub pre: Option<(String, Option<u64>)>,
    pub post: Option<(Option<String>, Option<u64>)>,
    pub dev: Option<(String, Option<u64>)>,
    pub local: Option<String>,
    pub has_wildcard: bool,
}

impl Version {
    /// Strict counterpart of [`Version::new`]: rejects what PEP 440 does not accept instead of salvaging it.
    pub fn parse(raw: &str) -> Option<Self> {
        let caps = PEP440.captures(raw.trim())?;
        let release = caps["release"]
            .split('.')
            .map(str::parse::<u64>)
            .collect::<Result<Vec<u64>, _>>()
            .ok()?;
        let pre = match caps.name("pre_l") {
            Some(label) => Some((label.as_str().to_ascii_lowercase(), optional_number(&caps, "pre_n")?)),
            None => None,
        };
        let post = if let Some(implicit) = caps.name("post_n1") {
            Some((Some("post".to_string()), Some(implicit.as_str().parse().ok()?)))
        } else if caps.name("post_l").is_some() {
            Some((Some("post".to_string()), optional_number(&caps, "post_n2")?))
        } else {
            None
        };
        let dev = if caps.name("dev").is_some() {
            Some(("dev".to_string(), optional_number(&caps, "dev_n")?))
        } else {
            None
        };
        Some(Self {
            // A zero epoch is dropped in the canonical form.
            epoch: optional_number(&caps, "epoch")?.filter(|epoch| *epoch != 0),
            release,
            pre,
            post,
            dev,
            local: caps.name("local").map(|local| normalize_local(local.as_str())),
            has_wildcard: false,
        })
    }

    pub fn new(raw: &str) -> Self {
        let mut input = raw.trim();
        let mut has_wildcard = false;
        if input.ends_with(".*") {
            has_wildcard = true;
            input = &input[..input.len() - 2];
        }

        let epoch = if let Some(idx) = input.find('!') {
            if let Ok(epoch) = input[..idx].parse() {
                input = &input[idx + 1..];
                Some(epoch)
            } else {
                None
            }
        } else {
            None
        };

        let local = if let Some(idx) = input.find('+') {
            let local = Some(input[idx + 1..].to_ascii_lowercase());
            input = &input[..idx];
            local
        } else {
            None
        };

        let mut main = input;
        let mut pre = None;
        let mut post = None;
        let mut dev = None;

        if let Some(idx) = main.to_ascii_lowercase().rfind("dev") {
            let (before, after) = main.split_at(idx);
            let after = &after[3..];
            let dev_num = Self::extract_number(after);
            dev = Some(("dev".to_string(), dev_num));
            main = Self::trim_separators_end(before);
        }

        if let Some(idx) = main.to_ascii_lowercase().rfind("post") {
            let (before, after) = main.split_at(idx);
            let after = &after[4..];
            let post_num = Self::extract_number(after);
            post = Some((Some("post".to_string()), post_num));
            main = Self::trim_separators_end(before);
        }

        // Labels are checked longest first so a shorter one is not matched inside a longer one ("rc" before "c").
        let pre_labels = ["preview", "alpha", "beta", "pre", "rc", "a", "b", "c"];
        for label in &pre_labels {
            if let Some(idx) = main.to_ascii_lowercase().rfind(label) {
                let (before, after) = main.split_at(idx);
                let after = &after[label.len()..];
                let pre_num = Self::extract_number(after);
                pre = Some((label.to_string(), pre_num));
                main = Self::trim_separators_end(before);
                break;
            }
        }

        let release: Vec<u64> = main.split('.').filter_map(|x| x.parse().ok()).collect();

        Self {
            epoch,
            release,
            pre,
            post,
            dev,
            local,
            has_wildcard,
        }
    }

    fn extract_number(s: &str) -> Option<u64> {
        let trimmed = s.trim_start_matches(['-', '_', '.']);
        let (num, _) = trimmed.split_at(trimmed.find(|c: char| !c.is_ascii_digit()).unwrap_or(trimmed.len()));
        if num.is_empty() { None } else { num.parse().ok() }
    }

    fn trim_separators_end(s: &str) -> &str {
        s.trim_end_matches(['-', '_', '.'])
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(epoch) = self.epoch {
            write!(f, "{}!", epoch)?;
        }
        for (i, part) in self.release.iter().enumerate() {
            if i > 0 {
                write!(f, ".")?;
            }
            write!(f, "{}", part)?;
        }
        if let Some((ref pre_l, pre_n)) = self.pre {
            f.write_str(match pre_l.as_str() {
                "alpha" | "a" => "a",
                "beta" | "b" => "b",
                "rc" | "c" | "pre" | "preview" => "rc",
                _ => pre_l,
            })?;
            if let Some(n) = pre_n {
                write!(f, "{}", n)?;
            } else {
                f.write_str("0")?;
            }
        }
        if let Some((ref _post_l, post_n)) = self.post {
            f.write_str(".post")?;
            if let Some(n) = post_n {
                write!(f, "{}", n)?;
            } else {
                f.write_str("0")?;
            }
        }
        if let Some((_, dev_n)) = self.dev {
            f.write_str(".dev")?;
            if let Some(n) = dev_n {
                write!(f, "{}", n)?;
            } else {
                f.write_str("0")?;
            }
        }
        if let Some(ref local) = self.local {
            f.write_str("+")?;
            f.write_str(&local.replace(['-', '_'], "."))?;
        }
        if self.has_wildcard {
            f.write_str(".*")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_unknown_pre_release_label() {
        let version = Version {
            epoch: None,
            release: vec![1, 2, 3],
            pre: Some(("unknown".to_string(), Some(1))),
            post: None,
            dev: None,
            local: None,
            has_wildcard: false,
        };
        assert_eq!(version.to_string(), "1.2.3unknown1");
    }

    #[test]
    fn test_display_unknown_pre_release_no_number() {
        let version = Version {
            epoch: None,
            release: vec![1, 2, 3],
            pre: Some(("xyz".to_string(), None)),
            post: None,
            dev: None,
            local: None,
            has_wildcard: false,
        };
        assert_eq!(version.to_string(), "1.2.3xyz0");
    }
}
