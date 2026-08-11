use harper_brill::UPOS;
use harper_core::{
	Document, Token,
	linting::{Lint, LintKind, Suggestion},
	spell::{Dictionary, FstDictionary},
};

use super::prose_words;
use crate::{
	ctx::Ctx,
	tags::Tags,
	wordset::{Entry, describe, equivalent, is_content},
};

/// Each wordset row restricts one (word, part of speech) pair, not the word: `work` is
/// unapproved as a verb and fine as a noun. Matching without the tag flags both.
pub fn unapproved_word(doc: &Document, tags: &Tags, ctx: &Ctx) -> Vec<Lint> {
	let src = doc.get_source();
	prose_words(doc, tags)
		.filter_map(|(i, t)| {
			if tags.immune(i) {
				return None;
			}
			let word = t.get_str(src).to_lowercase();
			if ctx.glossary.contains(&word) {
				return None;
			}
			let (row, entry) = match ctx.wordset.get(&word) {
				Some(entry) => (word.clone(), entry),
				None => inflected_row(ctx, t)?,
			};
			let used_as = tags.pos(i)?;
			if entry.approved || !equivalent(used_as, entry.pos?) {
				return None;
			}
			// An alternative equal to the word itself is how the wordset says "this word is
			// approved, but in another part of speech".
			let alternatives: Vec<&String> = ctx.wordset.alternatives(&row).iter().filter(|a| a.to_lowercase() != row).collect();
			// Some of the remaining ones are whole sentences of advice: worth reading, not applying.
			let replacements = alternatives.iter().filter(|a| !a.contains(' '));
			let named = match row == word {
				true => format!("`{word}`"),
				false => format!("`{word}` (`{row}`)"),
			};
			Some(Lint {
				span: t.span,
				lint_kind: LintKind::WordChoice,
				suggestions: replacements.map(|a| Suggestion::replace_with_match_case_str(a, t.span.get_content(src))).collect(),
				message: match alternatives.is_empty() {
					true => format!("{named} is not approved as {}; ASD-STE100 approves it only in another part of speech.", describe(used_as)),
					false => format!(
						"{named} is not approved as {}; use {}.",
						describe(used_as),
						alternatives.iter().map(|a| a.as_str()).collect::<Vec<_>>().join("; ")
					),
				},
				..Default::default()
			})
		})
		.collect()
}

pub fn wrong_pos(doc: &Document, tags: &Tags, ctx: &Ctx) -> Vec<Lint> {
	let src = doc.get_source();
	prose_words(doc, tags)
		.filter_map(|(i, t)| {
			if tags.immune(i) {
				return None;
			}
			let used_as = tags.pos(i)?;
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

/// openSTE has one row per lemma and no inflections, so a surface-form lookup is blind to most
/// of a real text. Harper's `derived_from` is derivational as well as inflectional
/// (`currently` → `current`), so the token's own form flags have to agree with the row's part of
/// speech before the row may speak for it.
fn inflected_row(ctx: &Ctx, t: &Token) -> Option<(String, Entry)> {
	let meta = t.kind.as_word()?.as_ref()?;
	let dictionary = FstDictionary::curated();
	let lemma: String = dictionary.get_word_from_id(&meta.derived_from?)?.iter().collect::<String>().to_lowercase();
	if ctx.glossary.contains(&lemma) {
		return None;
	}
	let entry = ctx.wordset.get(&lemma)?;
	let agrees = match entry.pos? {
		UPOS::VERB => meta.is_verb_past_form() || meta.is_verb_progressive_form() || meta.is_verb_third_person_singular_present_form(),
		UPOS::NOUN => meta.is_non_singular_noun(),
		_ => false,
	};
	agrees.then_some((lemma, entry))
}
