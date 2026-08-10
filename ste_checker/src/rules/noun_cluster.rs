use harper_brill::UPOS;
use harper_core::{
	Document, TokenStringExt,
	linting::{Lint, LintKind},
};

use super::{heading_spans, in_heading};
use crate::ctx::Ctx;

const MAX_NOUNS: usize = 3;

pub fn cluster(doc: &Document, _ctx: &Ctx) -> Vec<Lint> {
	let headings = heading_spans(doc);
	doc.iter_nominal_phrases()
		.filter_map(|np| {
			let span = np.span()?;
			if in_heading(&headings, span) {
				return None;
			}
			let nouns = np.iter().filter(|t| t.kind.is_upos(UPOS::NOUN)).count();
			if nouns <= MAX_NOUNS {
				return None;
			}
			Some(Lint {
				span,
				lint_kind: LintKind::Readability,
				message: format!("{nouns} nouns in a row; ASD-STE100 allows at most {MAX_NOUNS}. Break the cluster up with a preposition."),
				..Default::default()
			})
		})
		.collect()
}
