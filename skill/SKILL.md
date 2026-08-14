---
name: ste_checker
description: "What ste_checker reports on Markdown prose and what to do about a finding. Read when a dev shell or CI prints ASD-STE100 findings, and before writing docs/glossary.nix."
---

# ste_checker

Checks Markdown against the procedurally decidable part of ASD-STE100: the approved wordlist,
part of speech, sentence length, noun clusters, passive voice, compound tenses, `-ing` forms and
contractions. The other rules of the standard are semantic and are not checked. Findings are
warnings and the exit code is 0 unless `--deny`; `--format json` for a machine.

The standard is a whitelist of approximately 900 words, so a word your project owns is reported
until you declare it. Declare it in `docs/glossary.nix` as a Technical Name (approved as a noun)
or a Technical Verb (approved as a verb); declaring one does not declare the other. Write the
first file with:

```sh
ste_checker --suggest-glossary docs/.readme_assets/*.md > docs/glossary.nix
```

which lists every word outside the vocabulary and reads no glossary of its own.

Then edit it down. **Ordinary English never belongs in the glossary** — a finding on an ordinary
word is asking for the sentence to be written again, and absorbing the word instead disables the
whitelist one word at a time. Zero findings is not the target; the residual is prose debt.

Architecture and the reasoning behind each rule:
[ARCHITECTURE.md](https://github.com/valeratrades/ste_checker/blob/main/docs/ARCHITECTURE.md).
