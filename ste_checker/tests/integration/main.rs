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
			("unapproved-word", 44),
			("wrong-pos", 16),
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
			("unapproved-word", 35),
			("wrong-pos", 4),
			("sentence-length", 2),
			("noun-cluster", 0),
			("passive-voice", 5),
			("compound-tense", 0),
			("ing-verb", 9),
			("contraction", 2),
		]
	);
}

/// The headline tagger error: `cover` reads as a verb, which both invents a finding and splits
/// the nominal phrase the cluster rule used to be counted over.
#[test]
fn noun_cluster_survives_a_mistagged_stack() {
	let ctx = Ctx::new(AppConfig::default(), HashSet::new());
	let stack = "The main landing gear housing cover plate assembly bolt is loose.";
	assert_eq!(flagged(stack, "noun-cluster", &ctx).len(), 1);
	assert!(!flagged(stack, "unapproved-word", &ctx).contains(&"cover".to_string()));
	assert_eq!(flagged("Remove the wing tip fuel tank access panel.", "noun-cluster", &ctx).len(), 1);
}

/// Every one of these was a surviving `wrong-pos` finding on the corpus, and none of them is a
/// writing fault — the tagger and openSTE disagree about the tagset, not about the sentence.
#[test]
fn tagger_artifacts_are_not_wrong_pos() {
	let ctx = Ctx::new(AppConfig::default(), HashSet::new());
	for text in [
		"Standard logic that most providers share.",
		"When used as a lib, import with the toml.",
		"Delete the temporary files when the option is enabled.",
		"Auto-fix violations where possible.",
		"Run it on your local machine (where you can open a browser).",
		"Only a subset of rules know how to audit.",
		"Complete the OAuth authentication flow.",
	] {
		assert_eq!(flagged(text, "wrong-pos", &ctx), Vec::<String>::new(), "{text}");
	}
	assert_eq!(flagged("The report is a manual.", "wrong-pos", &ctx), vec!["manual"]);
}

/// The tokenizer splits hyphenated compounds, and the halves are not words anyone wrote.
#[test]
fn hyphen_compounds_do_not_reach_the_dictionary_rules() {
	let ctx = Ctx::new(AppConfig::default(), HashSet::new());
	assert_eq!(flagged("Argument-less pub fn new is the rule.", "wrong-pos", &ctx), Vec::<String>::new());
	assert_eq!(flagged("Wrap impl blocks in vim 1-fold markers.", "wrong-pos", &ctx), Vec::<String>::new());
	// The price of the rule: `well` is an unapproved adverb, and immunity cannot see that.
	assert_eq!(flagged("This is a well-known problem.", "unapproved-word", &ctx), Vec::<String>::new());
	assert_eq!(flagged("The system runs well.", "unapproved-word", &ctx), vec!["well"]);
}

/// `passive-voice` owns the span; the dictionary rules would only say the same thing again.
#[test]
fn passive_participle_is_reported_once() {
	let ctx = Ctx::new(AppConfig::default(), HashSet::new());
	let rules: Vec<&str> = ste_checker::check("Groups are specified by chat ID.", &ctx).iter().map(|f| f.rule).collect();
	assert_eq!(rules, vec!["passive-voice"]);
}

/// A sentence-initial word with a verb reading is only an imperative when nothing else in the
/// clause is finite.
#[test]
fn imperative_does_not_fire_on_a_finite_clause() {
	let ctx = Ctx::new(AppConfig::default(), HashSet::new());
	assert_eq!(flagged("Complete the OAuth authentication flow.", "wrong-pos", &ctx), Vec::<String>::new());
	assert_eq!(flagged("Complete documentation is required.", "wrong-pos", &ctx), vec!["Complete"]);
}

/// openSTE has one row per lemma and no inflections, so the surface form alone is blind to most
/// of a real text.
#[test]
fn inflections_inherit_their_lemma_row() {
	let ctx = Ctx::new(AppConfig::default(), HashSet::new());
	assert_eq!(flagged("The script requires a token.", "unapproved-word", &ctx), vec!["requires"]);
	// `quicker` derives from `quick`, an unapproved adjective, and is used as one — only the form
	// flags say it is a comparative rather than the row itself.
	assert!(!flagged("The build is quicker now.", "unapproved-word", &ctx).contains(&"quicker".to_string()));
	let glossed = Ctx::new(AppConfig::default(), HashSet::from(["require".to_string()]));
	assert_eq!(flagged("The script requires a token.", "unapproved-word", &glossed), Vec::<String>::new());
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

/// The words `rule` fired on, in source order. Spans are char indices into `text`.
fn flagged(text: &str, rule: &str, ctx: &Ctx) -> Vec<String> {
	let chars: Vec<char> = text.chars().collect();
	ste_checker::check(text, ctx)
		.iter()
		.filter(|f| f.rule == rule)
		.map(|f| chars[f.lint.span.start..f.lint.span.end].iter().collect())
		.collect()
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
