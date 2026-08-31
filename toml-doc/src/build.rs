//! Turning `toml_parser`'s flat event stream into the nested document.

use std::borrow::Cow;

use toml_parser::decoder::Encoding;
use toml_parser::parser::{Event, EventKind};

use crate::document::{Document, Entry, Header, Section, SectionKind, Trail};
use crate::trivia::{Comment, LineEnding, Pad, Padding, Piece, Trivia, Ws};
use crate::value::{Array, InlineTable, Key, KeyPart, KeyValue, Member, Quoting, Repr, Value};

pub struct Builder<'a, 'e> {
    source: &'a str,
    events: &'e [Event],
    at: usize,
    /// What a line the file left unclosed ends with.
    ending: LineEnding,
}

/// The event stream did not fit the document grammar, which happens only when the source failed to
/// parse; callers report the parse errors instead.
pub struct Malformed;

type Built<T> = Result<T, Malformed>;

impl<'a, 'e> Builder<'a, 'e> {
    pub fn new(source: &'a str, events: &'e [Event]) -> Self {
        // a line the file left unclosed still ends where it is held, so it takes the ending the
        // rest of the file uses
        let ending = if source.contains("\r\n") {
            LineEnding::Crlf
        } else {
            LineEnding::Lf
        };
        Self {
            source,
            events,
            at: 0,
            ending,
        }
    }

    pub fn document(mut self) -> Built<Document<'a>> {
        let mut document = Document::default();
        loop {
            let (lead, indent) = self.line_lead();
            let Some(kind) = self.peek() else {
                document.trailing = lead;
                if !indent.is_empty() {
                    // whitespace closing a file that has no final line break
                    document.trailing.push(Piece::Blank {
                        indent,
                        ending: self.ending,
                    });
                }
                return Ok(document);
            };
            match kind {
                EventKind::SimpleKey => {
                    let entry = self.entry(lead, indent)?;
                    match document.sections.last_mut() {
                        Some(section) => section.entries.push(entry),
                        None => document.root.push(entry),
                    }
                }
                EventKind::StdTableOpen => {
                    let header = self.header(SectionKind::Table, lead, indent)?;
                    document.sections.push(Section {
                        header,
                        entries: Vec::new(),
                    });
                }
                EventKind::ArrayTableOpen => {
                    let header = self.header(SectionKind::ArrayOfTables, lead, indent)?;
                    document.sections.push(Section {
                        header,
                        entries: Vec::new(),
                    });
                }
                _ => return Err(Malformed),
            }
        }
    }

    /// Consume the blank and comment lines before an item, returning them with the item's indent.
    fn line_lead(&mut self) -> (Trivia<'a>, Ws<'a>) {
        let mut lead = Trivia::default();
        let mut indent = Ws::default();
        let mut comment = None;
        while let Some(kind) = self.peek() {
            match kind {
                EventKind::Whitespace => indent = Ws::read(self.take_text()),
                EventKind::Comment => comment = Some(Comment::read(self.take_text())),
                EventKind::Newline => {
                    let ending = LineEnding::of(self.take_text());
                    lead.push(match comment.take() {
                        Some(text) => Piece::Comment { indent, text, ending },
                        None => Piece::Blank { indent, ending },
                    });
                    indent = Ws::default();
                }
                _ => break,
            }
        }
        if let Some(text) = comment {
            lead.push(Piece::Comment {
                indent,
                text,
                ending: self.ending,
            });
            indent = Ws::default();
        }
        (lead, indent)
    }

    fn entry(&mut self, lead: Trivia<'a>, indent: Ws<'a>) -> Built<Entry<'a>> {
        let key_value = self.key_value()?;
        Ok(Entry {
            lead,
            indent,
            key_value,
            trail: self.trail(),
        })
    }

    fn header(&mut self, kind: SectionKind, lead: Trivia<'a>, indent: Ws<'a>) -> Built<Header<'a>> {
        self.at += 1;
        let pre_key = self.spaces();
        let key = self.key()?;
        let post_key = self.spaces();
        match self.bump() {
            Some(EventKind::StdTableClose | EventKind::ArrayTableClose) => {}
            _ => return Err(Malformed),
        }
        Ok(Header {
            lead,
            indent,
            kind,
            pre_key,
            key,
            post_key,
            trail: self.trail(),
        })
    }

    fn key_value(&mut self) -> Built<KeyValue<'a>> {
        let key = self.key()?;
        let pre_eq = self.spaces();
        if self.bump() != Some(EventKind::KeyValSep) {
            return Err(Malformed);
        }
        let post_eq = self.spaces();
        Ok(KeyValue {
            key,
            pre_eq,
            post_eq,
            value: self.value()?,
        })
    }

    fn key(&mut self) -> Built<Key<'a>> {
        let mut parts = Vec::new();
        let mut lead = Ws::default();
        loop {
            let repr = self.simple_key()?;
            let before_spaces = self.at;
            let trail = self.spaces();
            if self.peek() != Some(EventKind::KeySep) {
                // spacing after the last segment belongs to whatever follows the key
                self.at = before_spaces;
                parts.push(KeyPart {
                    lead,
                    repr,
                    trail: Ws::default(),
                });
                return Ok(Key { parts });
            }
            self.at += 1;
            parts.push(KeyPart { lead, repr, trail });
            lead = self.spaces();
        }
    }

    fn simple_key(&mut self) -> Built<Repr<'a>> {
        let Some(event) = self
            .events
            .get(self.at)
            .filter(|event| event.kind() == EventKind::SimpleKey)
        else {
            return Err(Malformed);
        };
        self.at += 1;
        Repr::parsed(Cow::Borrowed(self.text_of(event)), quoting_of(event)).ok_or(Malformed)
    }

    fn value(&mut self) -> Built<Value<'a>> {
        match self.peek() {
            Some(EventKind::Scalar) => {
                let event = self.events[self.at];
                self.at += 1;
                let repr = Repr::parsed(Cow::Borrowed(self.text_of(&event)), quoting_of(&event));
                Ok(Value::Scalar(repr.ok_or(Malformed)?))
            }
            Some(EventKind::ArrayOpen) => {
                self.at += 1;
                self.array().map(Value::Array)
            }
            Some(EventKind::InlineTableOpen) => {
                self.at += 1;
                self.inline_table().map(Value::InlineTable)
            }
            _ => Err(Malformed),
        }
    }

    fn array(&mut self) -> Built<Array<'a>> {
        let mut array = Array::default();
        loop {
            let lead = self.padding();
            if self.peek() == Some(EventKind::ArrayClose) {
                self.at += 1;
                array.trailing = lead;
                return Ok(array);
            }
            let item = self.value()?;
            let (trail, comma, after) = self.close_member();
            // the comma the file wrote after this member closes the array only while it is the last
            // one, which is what the flag says once every member is read
            array.trailing_comma = comma;
            array.members.push(Member {
                lead,
                item,
                trail,
                after,
            });
        }
    }

    fn inline_table(&mut self) -> Built<InlineTable<'a>> {
        let mut table = InlineTable::default();
        loop {
            let lead = self.padding();
            if self.peek() == Some(EventKind::InlineTableClose) {
                self.at += 1;
                table.trailing = lead;
                return Ok(table);
            }
            let item = self.key_value()?;
            let (trail, comma, after) = self.close_member();
            table.trailing_comma = comma;
            table.members.push(Member {
                lead,
                item,
                trail,
                after,
            });
        }
    }

    /// What closes a member: the spacing and comment on its own line, whether a comma follows, and
    /// what shares the line with that comma.
    ///
    /// The comma may sit lines below the value it separates, so everything between them belongs to
    /// the member rather than to the one that comes next.
    fn close_member(&mut self) -> (Padding<'a>, bool, Padding<'a>) {
        let mut trail = self.same_line();
        let mark = self.at;
        let mut between = self.padding();
        if self.peek() != Some(EventKind::ValueSep) {
            self.at = mark;
            return (trail, false, Padding::default());
        }
        for part in between.parts_mut().drain(..) {
            trail.push(part);
        }
        self.at += 1;
        (trail, true, self.same_line())
    }

    /// What is written on the rest of this line: spacing, and the comment that closes it. A comment
    /// there is about the member just read rather than about whatever comes next, so it stays with
    /// it when members are reordered.
    fn same_line(&mut self) -> Padding<'a> {
        let mut padding = Padding::default();
        while self.peek() == Some(EventKind::Whitespace) {
            padding.push(Pad::Space(Ws::read(self.take_text())));
        }
        if self.peek() == Some(EventKind::Comment) {
            padding.push(Pad::Comment(Comment::read(self.take_text())));
        }
        padding
    }

    /// Whitespace, comments and line breaks inside an array or inline table.
    fn padding(&mut self) -> Padding<'a> {
        let mut padding = Padding::default();
        while let Some(kind) = self.peek() {
            let part = match kind {
                EventKind::Whitespace => Pad::Space(Ws::read(self.take_text())),
                EventKind::Comment => Pad::Comment(Comment::read(self.take_text())),
                EventKind::Newline => Pad::Newline(LineEnding::of(self.take_text())),
                _ => break,
            };
            padding.push(part);
        }
        padding
    }

    /// What closes a line: spacing, an optional comment, and the line break.
    fn trail(&mut self) -> Trail<'a> {
        let ws = self.spaces();
        let comment = (self.peek() == Some(EventKind::Comment)).then(|| Comment::read(self.take_text()));
        let ending = if self.peek() == Some(EventKind::Newline) {
            LineEnding::of(self.take_text())
        } else {
            self.ending
        };
        Trail { ws, comment, ending }
    }

    fn spaces(&mut self) -> Ws<'a> {
        if self.peek() == Some(EventKind::Whitespace) {
            Ws::read(self.take_text())
        } else {
            Ws::default()
        }
    }

    fn peek(&self) -> Option<EventKind> {
        self.events.get(self.at).map(Event::kind)
    }

    fn bump(&mut self) -> Option<EventKind> {
        let kind = self.peek()?;
        self.at += 1;
        Some(kind)
    }

    fn take_text(&mut self) -> &'a str {
        let text = self.events.get(self.at).map_or("", |event| self.text_of(event));
        self.at += 1;
        text
    }

    fn text_of(&self, event: &Event) -> &'a str {
        &self.source[event.span().start()..event.span().end()]
    }
}

fn quoting_of(event: &Event) -> Option<Quoting> {
    event.encoding().map(|encoding| match encoding {
        Encoding::BasicString => Quoting::Basic,
        Encoding::LiteralString => Quoting::Literal,
        Encoding::MlBasicString => Quoting::MlBasic,
        Encoding::MlLiteralString => Quoting::MlLiteral,
    })
}
