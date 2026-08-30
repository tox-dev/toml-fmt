//! Reordering and rewriting the members of an array.
//!
//! Each member owns the comments written above it and the spacing on either side of the comma that
//! follows it, while the array writes the commas between the members it ends up with. The
//! operations here are therefore ordinary `Vec` work: sorting moves a member's comments with it,
//! and removing one leaves the array as open or as closed as the file wrote it.

use std::cmp::Ordering;
use std::collections::HashSet;

use lexical_sort::natural_lexical_cmp;
use toml_doc::{Array, Member, Pad, Repr, Value};

use crate::group::is_group_marker;

/// Sort members within each `# Group:` block, holding the blocks in the order they were written.
///
/// A member the key function cannot read has nothing to sort by, and the array it sits in says
/// what it says by the order it was written in, so nothing moves.
pub fn sort<T, K, C>(array: &mut Array<'_>, to_key: &K, cmp: &C)
where
    K: Fn(&Member<'_, Value<'_>>) -> Option<T>,
    C: Fn(&T, &T) -> Ordering,
{
    let mut keys: Vec<T> = Vec::with_capacity(array.members.len());
    for member in &array.members {
        let Some(key) = to_key(member) else {
            return;
        };
        keys.push(key);
    }
    let groups = crate::group::member_ranges(&array.members);
    let mut held = std::mem::take(&mut array.members).into_iter().zip(keys);
    let mut placed: Vec<Member<'_, Value<'_>>> = Vec::with_capacity(held.len());
    for group in groups {
        let mut members: Vec<(Member<'_, Value<'_>>, T)> = held.by_ref().take(group.len()).collect();
        let Some((first, _)) = members.first_mut() else {
            continue;
        };
        // the marker names the group, not the member written under it, so it stays on top of it
        let marker = take_marker(first);
        members.sort_by(|(_, left), (_, right)| cmp(left, right));
        let at = placed.len();
        placed.extend(members.into_iter().map(|(member, _)| member));
        placed[at].lead.parts_mut().splice(0..0, marker);
    }
    array.members = placed;
}

/// [`sort`], where a member the key function cannot read holds the place the file gave it and the
/// ones it can read sort among the places that are left.
///
/// A generated entry names no one thing to sort by, and what it generates is read where it sits, so
/// it stays there while the names written around it move.
pub fn sort_placed<T, K, C>(array: &mut Array<'_>, to_key: &K, cmp: &C)
where
    K: Fn(&Member<'_, Value<'_>>) -> Option<T>,
    C: Fn(&T, &T) -> Ordering,
{
    let groups = crate::group::member_ranges(&array.members);
    let mut held = std::mem::take(&mut array.members).into_iter();
    let mut placed: Vec<Member<'_, Value<'_>>> = Vec::with_capacity(groups.iter().map(ExactSizeIterator::len).sum());
    for group in groups {
        placed.extend(placed_run(held.by_ref().take(group.len()).collect(), to_key, cmp));
    }
    array.members = placed;
}

fn placed_run<'a, T, K, C>(mut members: Vec<Member<'a, Value<'a>>>, to_key: &K, cmp: &C) -> Vec<Member<'a, Value<'a>>>
where
    K: Fn(&Member<'_, Value<'_>>) -> Option<T>,
    C: Fn(&T, &T) -> Ordering,
{
    let mut ranked: Vec<(usize, T)> = members
        .iter()
        .enumerate()
        .filter_map(|(at, member)| to_key(member).map(|key| (at, key)))
        .collect();
    if ranked.len() < 2 {
        return members;
    }
    // the marker names the group, not the member written under it, so it stays on top of it
    let marker = take_marker(&mut members[0]);
    let slots: Vec<usize> = ranked.iter().map(|(at, _)| *at).collect();
    ranked.sort_by(|(_, left), (_, right)| cmp(left, right));
    let mut moved: Vec<Option<Member<'a, Value<'a>>>> = members.into_iter().map(Some).collect();
    let taken: Vec<Member<'a, Value<'a>>> = ranked
        .iter()
        .map(|(at, _)| moved[*at].take().expect("each place is read once"))
        .collect();
    for (at, member) in slots.into_iter().zip(taken) {
        moved[at] = Some(member);
    }
    let mut placed: Vec<Member<'a, Value<'a>>> = moved
        .into_iter()
        .map(|held| held.expect("every place holds a member"))
        .collect();
    placed[0].lead.parts_mut().splice(0..0, marker);
    placed
}

/// Sort each run of members between the ones `stays` holds where they are.
///
/// A member whose place is part of what the file says, as an `include-group` is where the group it
/// pulls in belongs, keeps it; only what is written between two of those moves.
pub fn sort_runs<'a, T, S, K, C>(array: &mut Array<'a>, stays: &S, to_key: &K, cmp: &C)
where
    S: Fn(&Member<'_, Value<'_>>) -> bool,
    K: Fn(&Member<'_, Value<'_>>) -> Option<T>,
    C: Fn(&T, &T) -> Ordering,
{
    let trailing_comma = array.trailing_comma;
    let mut placed: Vec<Member<'a, Value<'a>>> = Vec::with_capacity(array.members.len());
    let mut run: Vec<Member<'a, Value<'a>>> = Vec::new();
    for member in std::mem::take(&mut array.members) {
        if stays(&member) {
            placed.append(&mut sorted_run(std::mem::take(&mut run), to_key, cmp));
            placed.push(member);
            continue;
        }
        run.push(member);
    }
    placed.append(&mut sorted_run(run, to_key, cmp));
    array.members = placed;
    array.trailing_comma = trailing_comma;
}

fn sorted_run<'a, T, K, C>(members: Vec<Member<'a, Value<'a>>>, to_key: &K, cmp: &C) -> Vec<Member<'a, Value<'a>>>
where
    K: Fn(&Member<'_, Value<'_>>) -> Option<T>,
    C: Fn(&T, &T) -> Ordering,
{
    let mut run = Array {
        members,
        trailing_comma: false,
        trailing: toml_doc::Padding::default(),
    };
    sort(&mut run, to_key, cmp);
    run.members
}

/// Sort a string array by the key each entry maps to.
pub fn sort_strings<K, C>(array: &mut Array<'_>, to_key: &K, cmp: &C)
where
    K: Fn(&str) -> String,
    C: Fn(&str, &str) -> Ordering,
{
    sort(
        array,
        &|member| string_of(member).map(|text| to_key(&text)),
        &|left: &String, right: &String| cmp(left, right),
    );
}

/// Drop later members whose key repeats an earlier one.
pub fn dedupe_strings<K>(array: &mut Array<'_>, to_key: &K)
where
    K: Fn(&str) -> String,
{
    let mut seen = HashSet::new();
    remove_members(array, |_, member| {
        string_of(member).is_none_or(|text| seen.insert(to_key(&text)))
    });
}

/// Drop the members whose text the predicate rejects.
pub fn retain_strings<P>(array: &mut Array<'_>, mut keep: P)
where
    P: FnMut(&str) -> bool,
{
    remove_members(array, |_, member| string_of(member).is_none_or(|text| keep(&text)));
}

/// Drop members, moving the comments a dropped member led with onto whatever follows it.
///
/// A comment closing the line above sits in the next member's lead, so removing that member without
/// this would take a comment about something else along with it. The commas are the array's, so
/// what is left keeps the trailing comma the file wrote.
fn remove_members<K>(array: &mut Array<'_>, mut keep: K)
where
    K: FnMut(usize, &Member<'_, Value<'_>>) -> bool,
{
    let last = array.members.len().saturating_sub(1);
    let mut last_kept = true;
    let mut carried: Vec<Pad<'_>> = Vec::new();
    let mut kept: Vec<Member<'_, Value<'_>>> = Vec::with_capacity(array.members.len());
    for (index, mut member) in std::mem::take(&mut array.members).into_iter().enumerate() {
        if keep(index, &member) {
            member.lead.parts_mut().splice(0..0, carried.drain(..));
            kept.push(member);
        } else {
            last_kept &= index != last;
            carry(&mut carried, &member);
        }
    }
    array.trailing.parts_mut().splice(0..0, carried);
    array.members = kept;
    // an array stays open on the comma that closes it, and dropping the member that comma followed
    // closes the array with it
    array.trailing_comma &= last_kept;
}

/// A dropped member's comments are about what the file still says around them, so they move to
/// whatever follows it. Each keeps a line of its own, since a comment runs to the end of its line
/// and would otherwise swallow the value written after it.
fn carry<'a>(carried: &mut Vec<Pad<'a>>, member: &Member<'a, Value<'a>>) {
    let comments = member
        .lead
        .parts()
        .iter()
        .chain(member.trail.parts())
        .chain(member.after.parts())
        .filter(|part| matches!(part, Pad::Comment(_)));
    for comment in comments {
        carried.push(comment.clone());
        carried.push(Pad::Newline(toml_doc::LineEnding::Lf));
    }
}

/// Rewrite the text of every string member.
///
/// Rewriting never drops a member: a formatter that cannot read what a file says must leave it
/// alone rather than delete it. [`retain_strings`] is what drops.
pub fn map_strings<F>(array: &mut Array<'_>, mut rewrite: F)
where
    F: FnMut(&str) -> String,
{
    for member in &mut array.members {
        let Some(text) = string_of(member) else {
            continue;
        };
        member.item = Value::Scalar(Repr::basic_string(&rewrite(&text)));
    }
}

/// The text a string member holds, or `None` when the member is not a string.
#[must_use]
pub fn string_of(member: &Member<'_, Value<'_>>) -> Option<String> {
    match &member.item {
        Value::Scalar(repr) if repr.quoting().is_some() => toml_doc::decode(repr).ok(),
        _ => None,
    }
}

/// Take the parts up to and including the group marker off the member's lead.
fn take_marker<'a>(member: &mut Member<'a, Value<'a>>) -> Vec<Pad<'a>> {
    let parts = member.lead.parts_mut();
    let last = parts.iter().rposition(|part| match part {
        Pad::Comment(text) => is_group_marker(text.as_ref()),
        _ => false,
    });
    last.map_or_else(Vec::new, |index| parts.drain(..=index).collect())
}

/// Sort a value when it is an array of strings, and leave it alone otherwise.
pub fn sort_strings_in<K, C>(value: &mut Value<'_>, to_key: &K, cmp: &C)
where
    K: Fn(&str) -> String,
    C: Fn(&str, &str) -> Ordering,
{
    if let Value::Array(array) = value {
        sort_strings(array, to_key, cmp);
    }
}

/// Sort a value that is a list of names, which is the order almost every list in a formatted file
/// reads in: what a name says rather than how it is capitalized, and a number by its value rather
/// than by its digits, so `py9` leads `py10`.
pub fn sort_names_in(value: &mut Value<'_>) {
    sort_strings_in(value, &|text| text.to_lowercase(), &|left, right| {
        natural_lexical_cmp(left, right)
    });
}

/// Drop repeats from a value when it is an array of strings.
pub fn dedupe_strings_in<K>(value: &mut Value<'_>, to_key: &K)
where
    K: Fn(&str) -> String,
{
    if let Value::Array(array) = value {
        dedupe_strings(array, to_key);
    }
}
