//! What holds items together: the `# Group:` markers sorting must not cross, and the name a
//! table groups under.

/// What opens a group, whatever case the comment writes it in.
const MARKER: &str = "group:";

/// Whether a comment opens a group, as `# Group: web`. Case does not matter.
#[must_use]
pub fn is_group_marker(comment: &str) -> bool {
    comment
        .trim_start()
        .strip_prefix('#')
        .map(str::trim_start)
        .and_then(|rest| rest.get(..MARKER.len()))
        .is_some_and(|head| head.eq_ignore_ascii_case(MARKER))
}

/// The ranges a `# Group:` marker splits the members into, or one range covering all of them.
///
/// A marker opens a group and holds it apart from the one before it, so nothing sorts across one.
pub fn member_ranges<T>(members: &[toml_doc::Member<'_, T>]) -> Vec<std::ops::Range<usize>> {
    let mut starts = vec![0];
    for (index, member) in members.iter().enumerate().skip(1) {
        let marked = member.lead.parts().iter().any(|part| match part {
            toml_doc::Pad::Comment(text) => is_group_marker(text),
            toml_doc::Pad::Space(_) | toml_doc::Pad::Newline(_) => false,
        });
        if marked {
            starts.push(index);
        }
    }
    starts.push(members.len());
    starts.windows(2).map(|pair| pair[0]..pair[1]).collect()
}

/// The name a table groups under: one level deeper for the nested prefixes, so `tool.black` and
/// `tool.ruff` are told apart. Ordering and spacing have to agree on this or a gap lands in the
/// wrong place.
///
/// The answer is the segments themselves, not the name they join into: joining first would make
/// `tool."a.b"` and `tool.a.b` the same table.
#[must_use]
pub fn base_segments(segments: &[String], nested_prefixes: &[&str]) -> Vec<String> {
    let head = segments.first().map_or("", String::as_str);
    let width = if nested_prefixes.contains(&head) { 2 } else { 1 };
    segments[..width.min(segments.len())].to_vec()
}
