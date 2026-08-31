//! The pieces a TOML formatter is built from: laying out a document, ordering its tables and
//! arrays, rewriting the strings inside it, and moving entries between nesting levels.

pub mod arrays;
pub mod build;
pub mod disabled;
pub mod group;
pub mod layout;
pub mod nesting;
pub mod pep508;
pub mod sections;
pub mod settings;
pub mod shape;
pub mod spacing;
pub mod strings;
pub mod width;

/// Read the source, run `format` over it, and hand back what that wrote.
///
/// This is the whole of what a formatter's entry point does either side of its own rules: the file
/// is read once, its disabled keys reach the rules as the keys they spell, and what comes out is
/// checked before the caller sees it.
///
/// # Errors
///
/// Returns where the source stops being a document, whatever `format` rejected it with, or where
/// the written text stops being a document.
pub fn formatted(
    content: &str,
    format: impl FnOnce(&mut toml_doc::Document<'_>) -> Result<(), String>,
) -> Result<String, String> {
    // what the file says is read here, where the file is still the one the caller wrote; the pass
    // below hands the formatter a document with its disabled keys turned back on, which may say the
    // same name twice
    let mut document = toml_doc::parse(content).map_err(|errors| errors[0].to_string())?;
    let written = disabled::try_with_disabled_keys(&mut document, content, format)?;
    written_document(&written)
}

/// Hand back what the formatter wrote, once it reads back as the document it is meant to be.
///
/// The file the caller holds is the last valid one until this returns, so text no parser accepts
/// never reaches it.
///
/// # Errors
///
/// Returns where the written text stops being a document.
pub fn written_document(written: &str) -> Result<String, String> {
    toml_doc::parse(written)
        .map(|_| written.to_owned())
        .map_err(|errors| rejected(&errors))
}

/// What to say about text the formatter wrote that no reader accepts.
pub(crate) fn rejected(errors: &[toml_doc::Error]) -> String {
    format!("the formatter wrote something no reader accepts: {}", errors[0])
}
