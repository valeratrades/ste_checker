use harper_core::{
	Document,
	linting::{Lint, LintKind, Suggestion},
};

use super::{pos, prose_words};
use crate::{
	ctx::Ctx,
	wordset::{describe, equivalent, is_content},
};

/// Each wordset row restricts one (word, part of speech) pair, not the word: `work` is
/// unapproved as a verb and fine as a noun. Matching without the tag flags both.
pub fn unapproved_word(doc: &Document, ctx: &Ctx) -> Vec<Lint> {
	let src = doc.get_source();
	prose_words(doc)
		.filter_map(|t| {
			let word = t.get_str(src).to_lowercase();
			if ctx.glossary.contains(&word) {
				return None;
			}
			let entry = ctx.wordset.get(&word)?;
			let used_as = pos(t)?;
			if entry.approved || !equivalent(used_as, entry.pos?) {
				return None;
			}
			// An alternative equal to the word itself is how the wordset says "this word is
			// approved, but in another part of speech".
			let alternatives: Vec<&String> = ctx.wordset.alternatives(&word).iter().filter(|a| a.to_lowercase() != word).collect();
			// Some of the remaining ones are whole sentences of advice: worth reading, not applying.
			let replacements = alternatives.iter().filter(|a| !a.contains(' '));
			Some(Lint {
				span: t.span,
				lint_kind: LintKind::WordChoice,
				suggestions: replacements.map(|a| Suggestion::replace_with_match_case_str(a, t.span.get_content(src))).collect(),
				message: match alternatives.is_empty() {
					true => format!("`{word}` is not approved as {}; ASD-STE100 approves it only in another part of speech.", describe(used_as)),
					false => format!(
						"`{word}` is not approved as {}; use {}.",
						describe(used_as),
						alternatives.iter().map(|a| a.as_str()).collect::<Vec<_>>().join("; ")
					),
				},
				..Default::default()
			})
		})
		.collect()
}

pub fn wrong_pos(doc: &Document, ctx: &Ctx) -> Vec<Lint> {
	let src = doc.get_source();
	prose_words(doc)
		.filter_map(|t| {
			let used_as = pos(t)?;
			if !is_content(used_as) {
				return None;
			}
			let word = t.get_str(src).to_lowercase();
			if ctx.glossary.contains(&word) {
				return None;
			}
			let entry = ctx.wordset.get(&word)?;
			if !entry.approved {
				return None; // `unapproved-word` already owns this token
			}
			let approved_as = entry.pos?;
			if equivalent(used_as, approved_as) {
				return None;
			}
			Some(Lint {
				span: t.span,
				lint_kind: LintKind::Usage,
				message: format!(
					"`{word}` is approved only as {}, but is used here as {}. Add it to the glossary if it is a Technical Name or Technical Verb.",
					describe(approved_as),
					describe(used_as)
				),
				..Default::default()
			})
		})
		.collect()
}
