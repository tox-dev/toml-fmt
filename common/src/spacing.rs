//! How many empty lines sit between one table and the next.
//!
//! Sections that belong to the same tool are held together; a change of tool gets a wider gap. The
//! gap goes above a section's leading comments, which belong to the section rather than to what
//! came before it.

use toml_doc::{Document, LineEnding, Piece, Trivia, Value};

use crate::group::base_segments;

/// How far apart tables are set.
#[derive(Debug, Clone, Copy)]
pub struct Spacing<'a> {
    /// Empty lines between tables of different tools.
    pub between_groups: usize,
    /// Empty lines between tables of the same tool, or `None` to leave them as written.
    pub within_group: Option<usize>,
    /// Prefixes whose tables group one level deeper, so `tool.ruff` and `tool.mypy` differ.
    pub nested_prefixes: &'a [&'a str],
    pub ending: LineEnding,
}

impl Spacing<'_> {
    /// Set the gap above every table but the first.
    pub fn apply(self, document: &mut Document<'_>) {
        // nothing precedes the first line, so a gap above it would be a gap above nothing
        let first = document
            .root
            .first_mut()
            .map(|entry| &mut entry.lead)
            .or_else(|| document.sections.first_mut().map(|section| &mut section.header.lead));
        if let Some(lead) = first {
            let kept = lead
                .pieces()
                .iter()
                .position(|piece| !piece.is_blank())
                .unwrap_or(lead.pieces().len());
            lead.pieces_mut().drain(..kept);
        }

        let groups: Vec<Vec<String>> = document
            .sections
            .iter()
            .map(|section| base_segments(&section.header.key.segments(), self.nested_prefixes))
            .collect();
        let names: Vec<Vec<String>> = document
            .sections
            .iter()
            .map(|section| section.header.key.segments())
            .collect();

        if !document.root.is_empty()
            && let Some(first) = document.sections.first_mut()
        {
            self.set_gap(&mut first.header.lead, self.between_groups);
        }
        for index in 1..document.sections.len() {
            // repeated `[[table]]` entries carry their own spacing, set where they are reordered
            if names[index] == names[index - 1] {
                continue;
            }
            let blanks = if groups[index] == groups[index - 1] {
                let Some(blanks) = self.within_group else { continue };
                blanks
            } else {
                self.between_groups
            };
            self.set_gap(&mut document.sections[index].header.lead, blanks);
        }
    }

    fn set_gap(self, lead: &mut Trivia<'_>, blanks: usize) {
        let pieces = lead.pieces_mut();
        let kept = pieces
            .iter()
            .position(|piece| !piece.is_blank())
            .unwrap_or(pieces.len());
        pieces.drain(..kept);
        let gap = std::iter::repeat_with(|| Piece::Blank {
            indent: "".into(),
            ending: self.ending,
        })
        .take(blanks);
        pieces.splice(0..0, gap);
    }
}

/// Cap every run of empty lines in the document at `max`.
///
/// The passes above set the gaps they know about; this one covers the rest, so a table no rule
/// recognizes, the root keys, and the end of the file all follow the same limit. It walks the
/// document's own trivia rather than its text, which is what keeps empty lines inside a multiline
/// string out of reach.
pub fn limit_blank_runs(document: &mut Document<'_>, max: usize) {
    for entry in &mut document.root {
        entry.lead.limit_blank_runs(max);
        limit_within(&mut entry.key_value.value, max);
    }
    for section in &mut document.sections {
        section.header.lead.limit_blank_runs(max);
        for entry in &mut section.entries {
            entry.lead.limit_blank_runs(max);
            limit_within(&mut entry.key_value.value, max);
        }
    }
    document.trailing.limit_blank_runs(max);
}

/// The same limit inside a value. A commented inline table keeps the spacing the file gave it, so
/// this is the only pass that reaches the empty lines it holds. A scalar's own text is what the
/// value says, so the walk stops there.
fn limit_within(value: &mut Value<'_>, max: usize) {
    match value {
        Value::Scalar(_) => {}
        Value::Array(array) => {
            for member in &mut array.members {
                member.lead.limit_blank_runs(max);
                member.trail.limit_blank_runs(max);
                member.after.limit_blank_runs(max);
                limit_within(&mut member.item, max);
            }
            array.trailing.limit_blank_runs(max);
        }
        Value::InlineTable(table) => {
            for member in &mut table.members {
                member.lead.limit_blank_runs(max);
                member.trail.limit_blank_runs(max);
                member.after.limit_blank_runs(max);
                limit_within(&mut member.item.value, max);
            }
            table.trailing.limit_blank_runs(max);
        }
    }
}
