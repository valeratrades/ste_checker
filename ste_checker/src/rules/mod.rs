use harper_brill::UPOS;
use harper_core::{Document, Span, Token, TokenStringExt, linting::Lint};

use crate::ctx::Ctx;

pub mod contraction;
pub mod dictionary;
pub mod noun_cluster;
pub mod sentence;
pub mod verb_form;

pub type Rule = fn(&Document, &Ctx) -> Vec<Lint>;

pub const RULES: &[(&str, Rule)] = &[
	("unapproved-word", dictionary::unapproved_word),
	("wrong-pos", dictionary::wrong_pos),
	("sentence-length", sentence::length),
	("noun-cluster", noun_cluster::cluster),
	("passive-voice", verb_form::passive),
	("compound-tense", verb_form::compound_tense),
	("ing-verb", verb_form::ing_as_verb),
	("contraction", contraction::contraction),
];

/// Section titles are not sentences, and in the README case `readme_fw` generates them.
pub(crate) fn heading_spans(doc: &Document) -> Vec<Span<char>> {
	doc.iter_headings().filter_map(|h| h.span()).collect()
}

pub(crate) fn in_heading(headings: &[Span<char>], span: Span<char>) -> bool {
	headings.iter().any(|h| h.contains(span.start))
}

pub(crate) fn prose_words(doc: &Document) -> impl Iterator<Item = &Token> {
	let headings = heading_spans(doc);
	doc.tokens().filter(move |t| t.kind.is_word() && !in_heading(&headings, t.span))
}

pub(crate) fn pos(t: &Token) -> Option<UPOS> {
	t.kind.as_word().and_then(|m| m.as_ref()).and_then(|m| m.pos_tag)
}
