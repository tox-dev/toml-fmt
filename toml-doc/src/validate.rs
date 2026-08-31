//! What a document says, as opposed to how it is written.
//!
//! The grammar accepts a file that names one table twice, or a day that no month has. Those are
//! documents no reader can build a value from, so a formatter must not rewrite them: it would hand
//! back a file that looks repaired while still saying nothing.

use std::collections::HashMap;

use std::ops::Range;

use crate::document::{Document, Entry, SectionKind};
use crate::value::{Key, Repr, Value};

/// Read the document as a value, reporting what it cannot say.
pub fn check(source: &str, document: &Document<'_>) -> Result<(), Vec<crate::Error>> {
    let mut root = Table {
        explicit: true,
        ..Table::default()
    };
    let mut report = Report {
        source,
        errors: Vec::new(),
    };
    for entry in &document.root {
        define(&mut root, &entry.key_value.key, &entry.key_value.value, &mut report);
    }
    for section in &document.sections {
        let Some(table) = open(&mut root, &section.header.key, section.header.kind, &mut report) else {
            continue;
        };
        for entry in &section.entries {
            define(table, &entry.key_value.key, &entry.key_value.value, &mut report);
        }
    }
    scalars(document, &mut report);
    if report.errors.is_empty() {
        Ok(())
    } else {
        Err(report.errors)
    }
}

/// What the document cannot say, and where in the source it says it.
struct Report<'s> {
    source: &'s str,
    errors: Vec<crate::Error>,
}

impl Report<'_> {
    fn says(&mut self, key: &Key<'_>, what: &str) {
        let message = format!("`{}` {what}", key.segments().join("."));
        let span = self.span_of(key.parts[0].repr.text(), key.parts[key.parts.len() - 1].repr.text());
        self.errors.push(crate::Error { message, span });
    }

    fn holds_no_value(&mut self, repr: &Repr<'_>) {
        let message = format!("`{}` is not a value TOML can read", repr.text());
        let span = self.span_of(repr.text(), repr.text());
        self.errors.push(crate::Error { message, span });
    }

    /// Everything a parsed document holds is a run of the source it was read from, so where a run
    /// sits is the distance between the two pointers.
    fn span_of(&self, from: &str, to: &str) -> Range<usize> {
        let at = |text: &str| {
            (text.as_ptr() as usize)
                .saturating_sub(self.source.as_ptr() as usize)
                .min(self.source.len())
        };
        at(from)..at(to).saturating_add(to.len()).min(self.source.len())
    }
}

#[derive(Default)]
struct Table {
    children: HashMap<String, Slot>,
    /// written as its own `[header]`, which may happen once
    explicit: bool,
    /// named by a dotted key, so a header may add tables under it but may not name it again
    dotted: bool,
    /// written as `{ ... }`, which closes it: nothing may add to what it already says
    inline: bool,
}

enum Slot {
    Table(Table),
    Array(Vec<Table>),
    Value,
}

impl Default for Slot {
    fn default() -> Self {
        Self::Table(Table::default())
    }
}

/// Walk a header's path, writing out the tables it names along the way.
fn open<'t>(root: &'t mut Table, key: &Key<'_>, kind: SectionKind, report: &mut Report<'_>) -> Option<&'t mut Table> {
    let segments = key.segments();
    let (last, path) = segments.split_last().expect("a key names at least one segment");
    let mut table = root;
    for segment in path {
        table = match table.children.entry(segment.clone()).or_default() {
            Slot::Table(held) if !held.inline => held,
            Slot::Array(elements) => elements.last_mut().expect("an array of tables has an element"),
            _ => {
                report.says(key, "is already something a table cannot extend");
                return None;
            }
        };
    }
    match kind {
        SectionKind::Table => match table.children.entry(last.clone()).or_default() {
            Slot::Table(held) if !held.explicit && !held.dotted && !held.inline => {
                held.explicit = true;
                Some(held)
            }
            _ => {
                report.says(key, "is defined twice");
                None
            }
        },
        SectionKind::ArrayOfTables => {
            let slot = table
                .children
                .entry(last.clone())
                .or_insert_with(|| Slot::Array(Vec::new()));
            let Slot::Array(elements) = slot else {
                report.says(key, "is already something an array of tables cannot extend");
                return None;
            };
            elements.push(Table {
                explicit: true,
                ..Table::default()
            });
            Some(elements.last_mut().expect("just pushed"))
        }
    }
}

/// Register a `key = value` in the table it was written under.
fn define(table: &mut Table, key: &Key<'_>, value: &Value<'_>, report: &mut Report<'_>) {
    let segments = key.segments();
    let (last, path) = segments.split_last().expect("a key names at least one segment");
    let mut table = table;
    for segment in path {
        // a table a dotted key reaches is closed to headers, but the key itself may extend it
        table = match table.children.entry(segment.clone()).or_insert_with(|| {
            Slot::Table(Table {
                dotted: true,
                ..Table::default()
            })
        }) {
            Slot::Table(held) if !held.explicit && !held.inline => held,
            _ => {
                report.says(key, "is already something a key cannot extend");
                return;
            }
        };
    }
    if table.children.contains_key(last) {
        report.says(key, "is written twice");
        return;
    }
    table.children.insert(last.clone(), slot_for(value, report));
}

/// What a value puts in the table: an inline table is a table nothing may extend, and anything else
/// is a value.
fn slot_for(value: &Value<'_>, report: &mut Report<'_>) -> Slot {
    let Value::InlineTable(inline) = value else {
        if let Value::Array(array) = value {
            for member in &array.members {
                let _ = slot_for(&member.item, report);
            }
        }
        return Slot::Value;
    };
    let mut table = Table {
        inline: true,
        ..Table::default()
    };
    for member in &inline.members {
        define(&mut table, &member.item.key, &member.item.value, report);
    }
    Slot::Table(table)
}

/// Every scalar the document holds, wherever it is written.
fn scalars(document: &Document<'_>, report: &mut Report<'_>) {
    for entry in &document.root {
        walk(entry, report);
    }
    for section in &document.sections {
        for entry in &section.entries {
            walk(entry, report);
        }
    }
}

fn walk(entry: &Entry<'_>, report: &mut Report<'_>) {
    fn visit(value: &Value<'_>, report: &mut Report<'_>) {
        match value {
            Value::Scalar(repr) => {
                if repr.quoting().is_none() && !is_value(repr.text()) {
                    report.holds_no_value(repr);
                }
            }
            Value::Array(array) => {
                for member in &array.members {
                    visit(&member.item, report);
                }
            }
            Value::InlineTable(table) => {
                for member in &table.members {
                    visit(&member.item.value, report);
                }
            }
        }
    }
    visit(&entry.key_value.value, report);
}

/// Whether the token spells a value. The grammar has said only that a bare token sits here, so a
/// number with no digits, or a day no month has, still reaches this far.
fn is_value(text: &str) -> bool {
    matches!(text, "true" | "false") || is_number(text) || is_moment(text)
}

/// What a number is written as. How wide the value is says nothing here: a document holds every
/// scalar as the text the file wrote, so a spelling this model reads back whole is one it keeps.
fn is_number(text: &str) -> bool {
    for (prefix, radix) in [("0x", 16), ("0o", 8), ("0b", 2)] {
        if let Some(digits) = text.strip_prefix(prefix) {
            return digit_run(digits, radix);
        }
    }
    let body = text.strip_prefix(['+', '-']).unwrap_or(text);
    matches!(body, "inf" | "nan") || is_decimal(body)
}

/// Whether the run is one or more digits of the radix, with `_` only ever between two of them.
fn digit_run(text: &str, radix: u32) -> bool {
    !text.is_empty()
        && !text.starts_with('_')
        && !text.ends_with('_')
        && !text.contains("__")
        && text.chars().all(|held| held == '_' || held.is_digit(radix))
}

/// An unsigned decimal integer, and the fraction and exponent a float adds to it.
fn is_decimal(text: &str) -> bool {
    let (whole, rest) = split_run(text);
    if !digit_run(whole, 10) || (whole.starts_with('0') && whole.len() > 1) {
        return false;
    }
    // how the fraction and the exponent are written is the grammar's to check; what is left here is
    // the shape around them, which is what tells a number from a date the grammar let through
    let rest = rest.strip_prefix('.').map_or(rest, |after| split_run(after).1);
    if rest.is_empty() {
        return true;
    }
    let Some(after) = rest.strip_prefix(['e', 'E']) else {
        return false;
    };
    let (exponent, rest) = split_run(after.strip_prefix(['+', '-']).unwrap_or(after));
    rest.is_empty() && digit_run(exponent, 10)
}

/// The leading run of digits and separators, and whatever follows it.
fn split_run(text: &str) -> (&str, &str) {
    text.split_at(
        text.find(|held: char| !held.is_ascii_digit() && held != '_')
            .unwrap_or(text.len()),
    )
}

/// A date, a time, or a date and a time with the offset one may carry.
fn is_moment(text: &str) -> bool {
    let Some(rest) = after_date(text) else {
        return after_time(text).is_some_and(str::is_empty);
    };
    if rest.is_empty() {
        return true;
    }
    let Some(after) = rest.strip_prefix(['T', 't', ' ']) else {
        return false;
    };
    let Some(rest) = after_time(after) else {
        return false;
    };
    rest.is_empty() || matches!(rest, "Z" | "z") || is_offset(rest)
}

/// `YYYY-MM-DD` and what follows it, where the day is one that month has.
fn after_date(text: &str) -> Option<&str> {
    let date = text.get(..10)?;
    let shaped = date.as_bytes()[4] == b'-'
        && date.as_bytes()[7] == b'-'
        && [0..4, 5..7, 8..10]
            .into_iter()
            .all(|span| date.as_bytes()[span].iter().all(u8::is_ascii_digit));
    (shaped && day_exists(date)).then(|| &text[10..])
}

fn day_exists(date: &str) -> bool {
    let (year, month, day) = (&date[0..4], &date[5..7], &date[8..10]);
    let (year, month, day) = (
        year.parse::<u16>().expect("four ascii digits"),
        month.parse::<u8>().expect("two ascii digits"),
        day.parse::<u8>().expect("two ascii digits"),
    );
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    day >= 1 && day <= days
}

/// `HH:MM`, the seconds and fraction it may carry, and what follows them.
fn after_time(text: &str) -> Option<&str> {
    let rest = two_digits(text, 24)?;
    let rest = two_digits(rest.strip_prefix(':')?, 60)?;
    let Some(after) = rest.strip_prefix(':') else {
        return Some(rest);
    };
    // 60 is the second a leap second is written as, which RFC 3339 gives `1990-12-31T23:59:60Z` for
    let rest = two_digits(after, 61)?;
    let Some(after) = rest.strip_prefix('.') else {
        return Some(rest);
    };
    let (fraction, rest) = split_run(after);
    (!fraction.is_empty() && !fraction.contains('_')).then_some(rest)
}

/// Two digits below the given bound, and what follows them.
fn two_digits(text: &str, less_than: u8) -> Option<&str> {
    let head = text.get(..2)?;
    if !head.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    (head.parse::<u8>().expect("two ascii digits") < less_than).then(|| &text[2..])
}

/// `+HH:MM`, whose hour RFC 3339 gives as 00 through 23.
fn is_offset(text: &str) -> bool {
    let Some(rest) = text.strip_prefix(['+', '-']) else {
        return false;
    };
    let Some(rest) = two_digits(rest, 24) else {
        return false;
    };
    rest.strip_prefix(':')
        .and_then(|after| two_digits(after, 60))
        .is_some_and(str::is_empty)
}
