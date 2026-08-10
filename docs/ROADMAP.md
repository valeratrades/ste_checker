# Roadmap

Baseline for everything below, over `tests/corpus/` (29 README assets, 680 tagged words):

```
                          no glossary    + starter glossary
 unapproved-word               56               29
 wrong-pos                     24               11
 ing-verb                       9                9
 passive-voice                  5                5
 sentence-length                2                2
 contraction                    2                2
 noun-cluster / compound-tense  0                0
                              ───              ───
                               98               58
```

Wordset coverage on the same corpus: **395 of 680 tagged words (58%) are in openSTE at all**.
Of the 207 distinct absent surface forms, **48 are plain inflections of a row we already have**
(`checks`, `created`, `files`, `enabled`, `needing`, `monitoring`…). Those are silent false
negatives — no rule can see them.

LanguageTool + TechScribe report P=0.86 / R=0.98. That is the number to chase.

## 0. Steal from LanguageTool — before anything ships downstream

Twenty years of production linguistics on us. Building ours first was to have concrete code to
diff against theirs. Read LT's English resources and the TechScribe STE customization side by
side with `src/rules/` and `src/wordset.rs`, then write a plan from what they do deeper.

- **`en/disambiguation.xml`** — the ordered, cascading, Brill-style disambiguator that runs
  *before* rule matching, each rule building on earlier ones. Highest-value item here. It is the
  direct answer to two measured problems:
  - the tagger's own errors: `housing cover` → `cover`=VERB, which both mislabels the word and
    splits the nominal phrase so `noun-cluster` never fires on a 5-noun stack;
  - **7 of the 11 surviving wrong-pos findings** are tagset artifacts, not writing faults —
    `when`/`where` tagged ADV against an approved CCONJ, `most`/`less`/`only` tagged ADV against
    an approved ADJ. `equivalent()` in `wordset.rs` is a hand-rolled approximation of what LT
    does properly; widening it by hand trades these for false negatives elsewhere.
- **Inflection / lemmatization** — the 58% coverage number above. openSTE ships `plural` empty
  for all 1951 rows and no verb forms, so this has to come from Harper's side: `IrregularVerbs`,
  `IrregularNouns`, `get_plurals`/`get_singulars`, `VerbFormFlags`. The parts exist; LT's
  morfologik full-form dictionaries show how they are assembled.
- **Sentence-boundary detection** — TechScribe names mis-segmentation as their primary error
  source, and every length rule depends on it. Table cells already arrive as one-word
  "sentences"; `MIN_WORDS` in `sentence.rs` is a guess, not a segmentation fix.
- **Chunking** — LT's post-disambiguation chunker against Harper's `iter_nominal_phrases`,
  specifically for noun-cluster counting (currently 0 hits on the corpus, which is suspicious).
- **TechScribe's STE-specific disambiguation** — e.g. `your` as an adjective per STE convention
  rather than a possessive pronoun. We hardcode that as an equivalence pair; they encode it in
  the disambiguator, where it belongs.
- **`grammar.xml` rule mechanics** — unification/agreement, exceptions, scope, and how they
  express the counting constraints Weir cannot.

Success measure: re-run the corpus calibration and show precision/recall movement against the
baseline above.

## 1. `readme_fw` integration

Pre-commit hook over `docs/.readme_assets/{usage,install*}.{md,typ}`, `severity: warning`, wired
at `v_flakes/flake.nix:91` alongside the existing `pre-commit-check`.

Two things to settle first:

- **The exit code is 0 by design**, so `pre-commit` hides the output — and the findings with it.
  A warning-severity hook that prints nothing is useless. Either the hook runs with `--deny` and
  the framework downgrades the failure, or it always prints.
- **`v_utils`' `Settings` derive prints `warning: no config file found` to stderr on every
  invocation**, deliberately and unconditionally. On a commit hook that is noise on every commit.
  Ship a config, pass `--config`, or drop the derive.

## 2. LLM target

A second pass invoking the `asd-ste100` skill (`~/.claude/skills/asd-ste100/`, MIT, vendored from
`danyuchn/asd-ste100-skill`) on files that already passed the procedural rules. The skill ships no
wordlist and works from STE's principles, so it composes with rather than duplicates the
dictionary rules. It covers what no checker can do procedurally: one-topic paragraphs, ellipsis,
list-for-sequences, topic sentences.

## 3. `.typ` support

`harper-typst` exists; Typst assets are currently unchecked.

## 4. `--suggest-glossary`

Emit the surviving wrong-pos and unapproved hits as a starter glossary instead of hand-writing one
per repo. `tests/corpus.glossary` took a human pass to keep honest — 23 words that name software
artifacts, against 29 surviving findings that are ordinary prose STE genuinely rejects (`need`,
`via`, `just`, `instead`, `normally`, `per`, `under`). A machine can propose the list; it cannot
draw that line.

## Not on the roadmap

**Closing the false-positive budget to zero.** The original plan expected `--deny` to exit 0 on
the corpus with a good glossary. It does not, and it should not: the residual is real ASD-STE100
debt in the text, not tool error. A glossary large enough to reach 0 would have to absorb ordinary
English, which is the one thing the glossary must never do.

**The remaining ~43 semantic rules.** They belong to item 2. This tool is a writing aid, not a
compliance certificate, and neither ASD nor STEMG endorse it.
