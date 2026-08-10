use std::{collections::HashSet, path::Path};

use crate::{
	config::AppConfig,
	wordset::{WORDSET, Wordset},
};

/// Everything the rules read besides the document itself.
pub struct Ctx {
	pub wordset: &'static Wordset,
	/// Per-project Technical Names and Technical Verbs, lowercased. STE permits these, and
	/// without them the dictionary rules bury a repo's own vocabulary in false positives.
	pub glossary: HashSet<String>,
	pub config: AppConfig,
}

impl Ctx {
	pub fn new(config: AppConfig, glossary: HashSet<String>) -> Self {
		Self {
			wordset: &WORDSET,
			glossary,
			config,
		}
	}

	/// One term per line; `#` starts a comment.
	pub fn read_glossary(path: &Path) -> std::io::Result<HashSet<String>> {
		Ok(std::fs::read_to_string(path)?
			.lines()
			.map(|l| l.split_once('#').map_or(l, |(before, _)| before).trim().to_lowercase())
			.filter(|l| !l.is_empty())
			.collect())
	}

	pub fn rule_enabled(&self, rule: &str) -> bool {
		!self.config.disable.iter().any(|d| d == rule)
	}
}
