Give all the files to one process. The language model loads one time, then each file
costs approximately 100 microseconds.

```sh
ste_checker docs/.readme_assets/usage.md docs/.readme_assets/installation.md
```

Findings are warnings, and the exit code is 0. To make them errors, add `--deny`. For
a machine-readable report, add `--format json`.

Your project has its own Technical Names and Technical Verbs, and ASD-STE100 tells you
to write them down. Put one word on each line in `docs/.ste_glossary`, or give another
path with `--glossary`. Words in the glossary are approved in each part of speech.

Set `text_type` to `description` in the configuration file to permit 25 words in a
sentence instead of 20. Put rule names in `disable` to switch rules off.
