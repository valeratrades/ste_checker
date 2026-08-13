# Technical Names and Technical Verbs a Rust/Nix command-line project may declare under
# ASD-STE100. Only words that name software artifacts or operations on them belong here —
# ordinary prose that STE wants rewritten does not.
#
# A Technical Name is approved as a noun and a Technical Verb as a verb, so a word the project
# uses both ways is declared twice.
{
  names = [
    "build"
    "channel"
    { name = "check"; desc = "one run of a lint rule over one file"; }
    "config"
    "default"
    "error"
    "false"
    "file"
    "flag"
    "function"
    "implementation"
    "input"
    "output"
    "release"
    "target"
    "true"
  ];

  verbs = [
    "build"
    "check"
    "copy"
    "delete"
    "enable"
    "generate"
    "restart"
    "start"
    "switch"
  ];
}
