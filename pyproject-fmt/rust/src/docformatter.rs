pub const KEY_ORDER: &[&str] = &[
    "in-place",
    "recursive",
    "check",
    "diff",
    "black",
    "pep257",
    "non-strict",
    "line-length",
    "wrap-summaries",
    "wrap-descriptions",
    "tab-width",
    "make-summary-multi-line",
    "close-quotes-on-newline",
    "pre-summary-newline",
    "pre-summary-multi-line",
    "pre-summary-space",
    "post-description-blank",
    "force-wrap",
    "line-range",
    "docstring-length",
    "non-cap",
    "exclude",
    "config",
];

/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(_key: &str) -> bool {
    false
}
