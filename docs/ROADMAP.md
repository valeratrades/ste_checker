# Roadmap

Baseline for everything below, over `ste_checker/tests/corpus/` (29 README assets, 680 tagged
words), before and after item 0:

```
                          no glossary        + starter glossary
                          before   after     before   after
 unapproved-word            56       44        29       35
 wrong-pos                  24       16        11        4
 ing-verb                    9        9         9        9
 passive-voice               5        5         5        5
 sentence-length             2        2         2        2
 contraction                 2        2         2        2
 noun-cluster                0        0         0        0
 compound-tense              0        0         0        0
                           ───      ───       ───      ───
                            98       78        58       57
```

`unapproved-word` going *up* under the glossary is the point: the lemma fallback made twelve
inflected forms visible (`requires`, `provides`, `discovers`, `discovered`, `fails`, `returning`,
`fixing`, `filtering`, `needing`, `decided`, `styles`, `supports`) that no rule could see before,
against six the immunize rules took away. `wrong-pos` falling
from 11 to 4 is the disambiguator. All four survivors were read by hand: `Order and group
dependencies` is a real STE finding, `manual fixing` is the deferred NOUN→ADJ case, and two are
new single-instance cascade artifacts (`Example config:` from the imperative rule, `macros use
inline` from the noun-run rule).

LanguageTool + TechScribe report P=0.86 / R=0.98. That is a direction, not a benchmark: it is an
unpublished vendor number with no corpus or protocol behind it.

## 0. Steal from LanguageTool — done

Two premises of the original entry did not survive contact, and are recorded here because they
constrain everything after.

**The content cannot be stolen.** `en/disambiguation.xml` is `Copyright (C) 2008 Marcin
Miłkowski` under LT's LGPL-2.1; this crate is BlueOak-1.0.0. TechScribe's STE customization is
closed and licence-keyed. Their tagset is Penn Treebank and openSTE is UPOS, so the rule bodies
would have needed translating regardless. What was taken is the *technique*: an ordered cascade,
`filter`/`immunize` actions, and marker scope — written against our own corpus.

**Retagging alone does not fix `noun-cluster`.** `iter_nominal_phrases` splits on `np_member`, a
second chunker output that the cascade does not touch, so fixing the tag would not have rejoined
the phrase. `noun_cluster` now counts maximal NOUN runs over `Tags` instead.

What shipped, in `src/tags.rs`:

- **Lemma fallback.** `DictWordMetadata::derived_from` + `Dictionary::get_word_from_id`, gated on
  the token's own form flags so a derivation (`quicker` → `quick`) cannot inherit its lemma's
  row, and checked against the glossary on the lemma as well as the surface form.
  `IrregularVerbs`/`IrregularNouns`/`get_plurals`/`get_singulars` were not needed — the measured
  misses are all affix-regular.
- **Two immunize rules.** Hyphenated-compound halves the tokenizer split (`Argument-less`,
  `1-fold`), and past participles governed by `be`, which `passive-voice` already owns. The price,
  asserted in the tests so it stays visible: every hyphenated compound is silenced, `well-known`
  included.
- **Four retagging rules** — imperative, subordinator, quantifier, noun run — in one
  left-to-right pass, one write per token, and only ever to a part of speech the curated
  dictionary already admits.
- **`equivalent()` pruned by measurement**, not by argument. Ten of eleven pairs suppressed
  nothing on the corpus once the cascade ran, but a pair is a claim about the two tagsets, not a
  corpus artifact; only `(ADP, ADV)` and `(ADV, ADP)` — particle versus preposition, which the
  cascade is the right layer for — were deleted.

Still open, each deferred for want of a second instance:

- `manual fixing` — NOUN→ADJ before a gerund. It is the only rule that would fight the noun-run
  rule, and there is one occurrence.
- `Example config:` — the imperative rule on a colon-terminated fragment with no finite verb.
- `macros use inline` — the noun-run rule on a plural subject followed by its own verb. Agreement
  is what LT's `grammar.xml` unification exists for.
- Sentence-boundary detection, LT's chunker, and `grammar.xml` unification (originally bullets 3,
  4 and 6). `MIN_WORDS` in `sentence.rs` is still a guess, but `sentence-length` finds 2 and
  `compound-tense` 0 on the corpus; there is no measured failure to chase yet.

## 1. `readme_fw` integration — shipped in v_flakes v1.6.80

`v_flakes.readme-fw`'s `shellHook` binstalls this crate from crates.io and runs it over
`docs/.readme_assets/{usage,install*}.md` on every dev shell entry, right after it writes
`README.md` from those same assets. Findings go to stderr; the exit code is ignored, so nothing
gates. Every `readme_fw` consumer gets it — repos pick it up as they bump their `v_flakes` pin.

Two things changed shape against the original plan:

- **Not a pre-commit hook.** `pre-commit` shows a hook's output only when the hook fails, so a
  warning-severity hook prints nothing — the one thing it exists to do. Nix prints the findings
  itself instead.
- **Config moved out of the config file.** `v_utils`' `Settings` derive prints `warning: no config
  file found` to stderr unconditionally and by design, which every consumer's shell entry would
  have carried. `v_utils` is gone; `text_type` and `disable` are plain clap flags that nix passes.

`.sh` assets are skipped — shell scripts are not prose. `.typ` waits on item 3.

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
per repo. `ste_checker/tests/corpus.glossary` took a human pass to keep honest — 23 words that
name software artifacts, against 35 surviving findings that are ordinary prose STE rejects (`need`,
`via`, `just`, `instead`, `normally`, `per`, `under`). A machine can propose the list; it cannot
draw that line.

## Not on the roadmap

**Closing the false-positive budget to zero.** The original plan expected `--deny` to exit 0 on
the corpus with a good glossary. It does not, and it should not: the residual is real ASD-STE100
debt in the text, not tool error. A glossary large enough to reach 0 would have to absorb ordinary
English, which is the one thing the glossary must never do.

**The remaining ~43 semantic rules.** They belong to item 2. This tool is a writing aid, not a
compliance certificate, and neither ASD nor STEMG endorse it.
