//! The shape a formatted file takes: which tables fold into their parents, and what the layout
//! writes once every rule has run.
//!
//! Both formatters answer these the same way, so the answer is written once and each of them says
//! only what is its own: the tables it nests under, and the settings a user gave it.

use std::collections::HashSet;

use toml_doc::{Document, LineEnding};

/// Which tables fold into their parent.
pub struct Tables {
    /// What a table does where no setting names it or anything above it.
    pub default_collapse: bool,
    pub expand: HashSet<Vec<String>>,
    pub collapse: HashSet<Vec<String>>,
}

impl Tables {
    /// Read the settings a user wrote, each naming a table the way TOML names one: `a."b.c"` is two
    /// segments and `a.b.c` three, so the two name different tables and select them apart.
    #[must_use]
    pub fn new(table_format: &str, expand: &[String], collapse: &[String]) -> Self {
        Self {
            default_collapse: table_format == "short",
            expand: expand.iter().map(|name| crate::sections::parse_name(name)).collect(),
            collapse: collapse.iter().map(|name| crate::sections::parse_name(name)).collect(),
        }
    }

    /// Whether the table folds into its parent, per the closest setting that names it or one of the
    /// tables above it. The name is compared segment by segment, so a setting cannot cut a quoted
    /// name holding a dot in half.
    #[must_use]
    pub fn should_collapse(&self, table_name: &[String]) -> bool {
        for depth in (1..=table_name.len()).rev() {
            let name = &table_name[..depth];
            if self.collapse.contains(name) {
                return true;
            }
            if self.expand.contains(name) {
                return false;
            }
        }
        self.default_collapse
    }
}

/// What the layout writes once every rule has run.
pub struct Written<'a> {
    pub column_width: usize,
    pub indent: usize,
    /// The blank lines between one root table and the next.
    pub separate_root_table: &'a str,
    /// The blank lines between the sub-tables of one table, where they are written out.
    pub sub_table_spacing: &'a str,
    pub table_format: &'a str,
    /// The keys whose values carry meaning that line breaks would obscure.
    pub skip_wrap_for_keys: &'a [String],
    /// The tables whose children are written under them rather than beside them.
    pub nested_prefixes: &'a [&'a str],
}

impl Written<'_> {
    /// Wrap what runs past the column, lay out every line, line up the comments, and space the
    /// tables apart.
    pub fn apply(&self, document: &mut Document<'_>) {
        crate::strings::wrap_long_strings(document, self.column_width, self.indent, self.skip_wrap_for_keys);
        crate::layout::Layout {
            column_width: self.column_width,
            indent: self.indent,
            ending: LineEnding::Lf,
        }
        .apply(document);
        crate::layout::align_array_comments(document);
        crate::spacing::Spacing {
            between_groups: self.separate_root_table.matches('\n').count(),
            within_group: (self.table_format == "long").then(|| self.sub_table_spacing.matches('\n').count()),
            nested_prefixes: self.nested_prefixes,
            ending: LineEnding::Lf,
        }
        .apply(document);
        // a run of blank lines the file wrote reads as one gap, and nothing needs a second
        crate::spacing::limit_blank_runs(document, 2);
    }
}
