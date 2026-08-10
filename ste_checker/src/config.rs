use serde::{Deserialize, Serialize};
use v_utils::macros as v_macros;

#[derive(Clone, Debug, Default, v_macros::MyConfigPrimitives, v_macros::Settings)]
pub struct AppConfig {
	#[serde(default)]
	pub text_type: TextType,
	/// Rule names to switch off entirely; `--help` lists them.
	#[serde(default)]
	pub disable: Vec<String>,
}

/// ASD-STE100 allows longer sentences in descriptive text than in procedures.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TextType {
	#[default]
	#[serde(alias = "procedural")]
	Procedure,
	#[serde(alias = "descriptive")]
	Description,
}

impl TextType {
	pub fn max_sentence_words(self) -> usize {
		match self {
			Self::Procedure => 20,
			Self::Description => 25,
		}
	}
}
