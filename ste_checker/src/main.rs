use std::{
	collections::HashSet,
	path::{Path, PathBuf},
};

use clap::Parser;
use color_eyre::eyre::{Result, WrapErr, bail};
use ste_checker::{
	Ctx,
	config::{AppConfig, SettingsFlags},
	report,
};

const DEFAULT_GLOSSARY: &str = "docs/.ste_glossary";

const SCOPE: &str = "\
Checks Markdown against the procedurally decidable part of ASD-STE100: the approved
wordlist, part-of-speech restrictions, sentence length, noun clusters, passive voice,
compound tenses, -ing forms and contractions. The other ~43 rules of the standard are
semantic and are not checked.

Findings are warnings; the exit code stays 0 unless --deny is given. Neither ASD nor
STEMG endorse this tool, and it certifies nothing.

Words your project owns (Technical Names and Technical Verbs) belong in the glossary
file, one per line. ASD-STE100 provides for exactly that list.";

#[derive(Parser)]
#[command(author, version = concat!(env!("CARGO_PKG_VERSION"), " (", env!("GIT_HASH"), ")"), about, long_about = SCOPE)]
struct Cli {
	/// Markdown files to check. Pass them all at once: model load dominates per-file cost.
	#[arg(required = true)]
	paths: Vec<PathBuf>,
	/// Exit 1 when anything is found.
	#[arg(long)]
	deny: bool,
	#[arg(long, value_enum, default_value_t = Format::Human)]
	format: Format,
	/// Approved project terms, one per line. Defaults to `docs/.ste_glossary` when present.
	#[arg(long)]
	glossary: Option<PathBuf>,
	#[command(flatten)]
	settings: SettingsFlags,
}

#[derive(Clone, Copy, PartialEq, clap::ValueEnum)]
enum Format {
	Human,
	Json,
}

fn main() -> Result<()> {
	color_eyre::install()?;
	let cli = Cli::parse();
	let config = AppConfig::try_build(cli.settings).wrap_err("failed to read config")?;
	for rule in &config.disable {
		if !ste_checker::rule_names().any(|r| r == rule) {
			bail!("`{rule}` is not a rule; known rules: {}", ste_checker::rule_names().collect::<Vec<_>>().join(", "));
		}
	}

	let glossary = match &cli.glossary {
		Some(path) => Ctx::read_glossary(path).wrap_err_with(|| format!("failed to read glossary {}", path.display()))?,
		None => {
			let path = Path::new(DEFAULT_GLOSSARY);
			match path.exists() {
				true => Ctx::read_glossary(path).wrap_err_with(|| format!("failed to read glossary {}", path.display()))?,
				false => HashSet::new(),
			}
		}
	};
	let ctx = Ctx::new(config, glossary);

	let mut json = Vec::new();
	let mut found = 0usize;
	for path in &cli.paths {
		let display = path.display().to_string();
		let text = std::fs::read_to_string(path).wrap_err_with(|| format!("failed to read {display}"))?;
		let findings = ste_checker::check(&text, &ctx);
		found += findings.len();
		match cli.format {
			Format::Human =>
				if let Some(report) = report::human(&display, &text, &findings) {
					eprintln!("{report:?}");
				},
			Format::Json => json.extend(report::json(&display, &text, &findings).into_iter().map(|f| serde_json::to_value(f).expect("plain data"))),
		}
	}
	if cli.format == Format::Json {
		println!("{}", serde_json::to_string_pretty(&json)?);
	}

	if cli.deny && found > 0 {
		std::process::exit(1);
	}
	Ok(())
}
