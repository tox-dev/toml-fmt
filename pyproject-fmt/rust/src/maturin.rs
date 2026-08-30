pub const KEY_ORDER: &[&str] = &[
    "module-name",
    "bindings",
    "python-source",
    "python-packages",
    "python-bin-path",
    "src",
    "manifest-path",
    "include",
    "exclude",
    "sdist-generator",
    "data",
    "features",
    "no-default-features",
    "all-features",
    "rustc-args",
    "unstable-flags",
    "config",
    "profile",
    "target",
    "target-dir",
    "compatibility",
    "auditwheel",
    "skip-auditwheel",
    "strip",
    "include-import-lib",
    "frozen",
    "locked",
    "offline",
    "zig",
    "use-cross",
    "use-base-python",
];

// maturin compiles `exclude` into an ordered override program, where a later pattern wins and a
// leading `!` takes back what an earlier one matched, so it keeps the order it was written in, and
// `rustc-args` and `unstable-flags` are argv
const SORT_ARRAYS: &[&str] = &["python-packages", "include", "features"];

/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(key: &str) -> bool {
    SORT_ARRAYS.contains(&key)
}
