//! The single integration binary, per <https://matklad.github.io/2021/02/27/delete-cargo-integration-tests.html>.
//!
//! `tests/corpus/` is every `usage*.md` and `installation*.md` under `~/s/*/docs/.readme_assets/`
//! at the time of writing — the text this checker was calibrated against. The counts below are a
//! snapshot, not a target: any change to the wordset, the POS-equivalence table or a rule moves
//! them, and that movement is the regression signal.
use std::{collections::HashSet, path::Path};

use ste_checker::{Ctx, config::AppConfig};

const CORPUS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/corpus");
const GLOSSARY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/corpus.glossary");

#[test]
fn corpus_calibration() {
	let counts = corpus_counts(HashSet::new());
	assert_eq!(
		counts,
		vec![
			("unapproved-word", 56),
			("wrong-pos", 24),
			("sentence-length", 2),
			("noun-cluster", 0),
			("passive-voice", 5),
			("compound-tense", 0),
			("ing-verb", 9),
			("contraction", 2),
		]
	);
}

/// What a project's own Technical Names and Technical Verbs buy back. The remainder is real
/// ASD-STE100 debt in the corpus, so `--deny` still exits non-zero over it.
#[test]
fn glossary_absorbs_technical_vocabulary() {
	let counts = corpus_counts(Ctx::read_glossary(Path::new(GLOSSARY)).unwrap());
	assert_eq!(
		counts,
		vec![
			("unapproved-word", 29),
			("wrong-pos", 11),
			("sentence-length", 2),
			("noun-cluster", 0),
			("passive-voice", 5),
			("compound-tense", 0),
			("ing-verb", 9),
			("contraction", 2),
		]
	);
}

/// The case the standard is built around, and the one that needs a live POS tagger: the same
/// spelling is approved in one part of speech and not in another.
#[test]
fn part_of_speech_decides_approval() {
	let ctx = Ctx::new(AppConfig::default(), HashSet::new());
	let flagged = |text: &str| ste_checker::check(text, &ctx).iter().any(|f| f.rule == "unapproved-word");
	assert!(flagged("Do not work the lever."));
	assert!(!flagged("The work is done."));
}

/// Harper counts characters; miette and editors count bytes.
#[test]
fn reported_offsets_index_bytes() {
	let ctx = Ctx::new(AppConfig::default(), HashSet::new());
	let text = "Ünicode — do not work the lever.\n";
	let findings = ste_checker::check(text, &ctx);
	let reported = ste_checker::report::json("t.md", text, &findings);
	let word = reported.iter().find(|f| f.rule == "unapproved-word").expect("`work` as a verb is unapproved");
	assert_eq!(&text[word.start..word.end], "work");
	assert!(ste_checker::report::human("t.md", text, &findings).is_some());
}

fn corpus_counts(glossary: HashSet<String>) -> Vec<(&'static str, usize)> {
	let ctx = Ctx::new(AppConfig::default(), glossary);
	let mut paths: Vec<_> = std::fs::read_dir(CORPUS).unwrap().map(|e| e.unwrap().path()).collect();
	paths.sort();
	assert_eq!(paths.len(), 29, "corpus changed size; recalibrate before touching these numbers");

	let mut counts: Vec<(&'static str, usize)> = ste_checker::rule_names().map(|r| (r, 0)).collect();
	for path in paths {
		for finding in ste_checker::check(&std::fs::read_to_string(&path).unwrap(), &ctx) {
			counts.iter_mut().find(|(rule, _)| *rule == finding.rule).expect("rule names come from the same table").1 += 1;
		}
	}
	counts
}
