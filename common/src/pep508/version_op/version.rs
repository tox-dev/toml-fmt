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
    pub fn new(raw: &str) -> Self {
        let mut input = raw.trim();
        let mut has_wildcard = false;
        if input.ends_with(".*") {
            has_wildcard = true;
            input = &input[..input.len() - 2];
        }

        // Parse epoch (e.g., "1!" from "1!2.0.0")
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

        // Parse local version (e.g., "+local" from "1.0.0+local")
        let local = if let Some(idx) = input.find('+') {
            let local = Some(input[idx + 1..].to_ascii_lowercase());
            input = &input[..idx];
            local
        } else {
            None
        };

        // Parse pre/post/dev releases
        let mut main = input;
        let mut pre = None;
        let mut post = None;
        let mut dev = None;

        // Parse dev release (e.g., ".dev3" or "dev3")
        if let Some(idx) = main.to_ascii_lowercase().rfind("dev") {
            let (before, after) = main.split_at(idx);
            let after = &after[3..];
            let dev_num = Self::extract_number(after);
            dev = Some(("dev".to_string(), dev_num));
            main = Self::trim_separators_end(before);
        }

        // Parse post release (e.g., ".post2" or "post2")
        if let Some(idx) = main.to_ascii_lowercase().rfind("post") {
            let (before, after) = main.split_at(idx);
            let after = &after[4..];
            let post_num = Self::extract_number(after);
            post = Some((Some("post".to_string()), post_num));
            main = Self::trim_separators_end(before);
        }

        // Parse pre-release (e.g., "a1", "beta2", "rc3")
        // Labels checked longest first to avoid partial matches (e.g. "rc" before "c")
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

        // Parse release numbers (e.g., "1.2.3" -> [1, 2, 3])
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
        if num.is_empty() {
            None
        } else {
            num.parse().ok()
        }
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
        // Release segment
        for (i, part) in self.release.iter().enumerate() {
            if i > 0 {
                write!(f, ".")?;
            }
            write!(f, "{}", part)?;
        }
        // Pre-release
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
        // Post-release
        if let Some((ref _post_l, post_n)) = self.post {
            f.write_str(".post")?;
            if let Some(n) = post_n {
                write!(f, "{}", n)?;
            } else {
                f.write_str("0")?;
            }
        }
        // Dev-release
        if let Some((_, dev_n)) = self.dev {
            f.write_str(".dev")?;
            if let Some(n) = dev_n {
                write!(f, "{}", n)?;
            } else {
                f.write_str("0")?;
            }
        }
        // Local
        if let Some(ref local) = self.local {
            f.write_str("+")?;
            f.write_str(&local.replace(['-', '_'], "."))?;
        }
        // Append .*, if this version had a wildcard
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
