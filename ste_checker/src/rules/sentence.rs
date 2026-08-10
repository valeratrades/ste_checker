use harper_core::{
	Document, TokenStringExt,
	linting::{Lint, LintKind},
};

use super::{heading_spans, in_heading};
use crate::ctx::Ctx;

/// Table cells and list fragments each parse as their own one- or two-word "sentence".
const MIN_WORDS: usize = 3;

pub fn length(doc: &Document, ctx: &Ctx) -> Vec<Lint> {
	let headings = heading_spans(doc);
	let max = ctx.config.text_type.max_sentence_words();
	doc.iter_sentences()
		.filter_map(|s| {
			let span = s.span()?;
			if in_heading(&headings, span) {
				return None;
			}
			let words = s.iter().filter(|t| t.kind.is_word()).count();
			if words < MIN_WORDS || words <= max {
				return None;
			}
			Some(Lint {
				span,
				lint_kind: LintKind::Readability,
				message: format!("{words} words; ASD-STE100 allows at most {max} here. Split the sentence."),
				..Default::default()
			})
		})
		.collect()
}
