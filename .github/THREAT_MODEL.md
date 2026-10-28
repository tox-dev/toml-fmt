# Threat model

This model says what toml-fmt trusts, where untrusted input enters, and which guarantees hold at each boundary, so a
report can be classified as a vulnerability or a bug. [SECURITY.md](SECURITY.md) covers reporting; if the two disagree,
SECURITY.md wins and we fix this file.

## How toml-fmt works

toml-fmt reads each TOML file named on the command line or standard input, parses it with the Rust `toml_parser` crate,
and writes the formatted text back to the same path, to standard output, or reports a diff with `--check`. It runs no
code from the file, makes no network requests, and touches no other file. It typically runs as a pre-commit hook, often
on pre-commit.ci or in CI against content from pull requests.

Both tools normalize content as well as layout: pyproject-fmt sorts and normalizes dependency specifiers and
classifiers. A normalization that changes which package or version a specifier selects is a correctness bug.

```mermaid
flowchart LR
    user["User, options<br/>(trusted)"] --> fmt["toml-fmt"]
    file[("TOML file<br/>(untrusted content)")] -->|read| fmt
    fmt -->|"formatted text to the same path,<br/>stdout, or a diff"| file
    classDef trusted fill:#cfe4ff,stroke:#1f6feb,color:#0b1f3a;
    classDef boundary fill:#ffe3a3,stroke:#d29200,color:#3a2c00;
    class user,fmt trusted;
    class file boundary;
```

## Compared with similar tools

We drew the boundary where comparable projects in other ecosystems draw theirs. The shared rule, stated by the
[Node.js threat model](https://github.com/nodejs/node/blob/main/SECURITY.md#the-nodejs-threat-model), is that a report
which first needs a trusted element compromised is out of scope; the first triage question, as
[DOMPurify](https://github.com/cure53/DOMPurify/blob/main/SECURITY.md#what-we-consider-a-vulnerability) puts it, is what
the attacker controls.

| Project                                                          | Language   | Where it draws the line                                                                                                          |
| ---------------------------------------------------------------- | ---------- | -------------------------------------------------------------------------------------------------------------------------------- |
| [Taplo](https://github.com/tamasfe/taplo)                        | Rust       | TOML formatter and language server; publishes no security policy, so it offers no line to compare beyond treating input as data. |
| [dprint](https://github.com/dprint/dprint/security/policy)       | Rust       | Formatter platform; patches only the latest release and takes reports privately.                                                 |
| [Prettier](https://github.com/prettier/prettier/security/policy) | JavaScript | Formats untrusted source; reports go through Tidelift, and the policy defines no further scope.                                  |

## Trusted

An attacker who controls any of these already has what toml-fmt could give them:

- The user, the command-line options and the list of files to format.
- The Python runtime and the environment the hook runs in.

## Untrusted inputs

| Input                                              | Comes from                         | What toml-fmt guarantees                                                             |
| -------------------------------------------------- | ---------------------------------- | ------------------------------------------------------------------------------------ |
| Contents of each TOML file                         | Anyone who can open a pull request | Treated as data: parsed and printed, never evaluated or used to choose another path. |
| `[tool.pyproject-fmt]` and `[tox-toml-fmt]` tables | The repository being formatted     | Only change layout choices such as width, indentation and ordering.                  |

## Threats and controls

| STRIDE                 | Threat                                                      | Control                                                          |
| ---------------------- | ----------------------------------------------------------- | ---------------------------------------------------------------- |
| Tampering              | A crafted file makes the formatter write somewhere else.    | The output path is the input path; content never selects a path. |
| Information disclosure | Formatting pulls in another file's content.                 | The Rust `toml_parser` crate resolves no external references.    |
| Denial of service      | A short input triggers exponential parsing or a hang in CI. | Treated as a vulnerability when the input is small; report it.   |

## Release chain

Releases run on the shared [tox-dev/github-workflows](https://github.com/tox-dev/github-workflows) pipeline. A release
App, not a person, pushes the release commit and tag; the build installs its tools hash-checked from `pylock.toml` with
a seven-day upload cooldown; PyPI receives the files through
[trusted publishing](https://docs.pypi.org/trusted-publishers/) with attestations, and the GitHub Release is immutable.
Every workflow pins third-party actions to a commit SHA.

## Vulnerability or bug

| Report                                                                         | Verdict             | Why                                       |
| ------------------------------------------------------------------------------ | ------------------- | ----------------------------------------- |
| A crafted `pyproject.toml` makes pyproject-fmt write another file              | Vulnerability       | Content must never choose an output path. |
| A 1 KB file with deep nesting hangs the Rust parser                            | Vulnerability       | Small-input hang in CI.                   |
| pyproject-fmt reorders `dependencies` so a marker applies to the wrong package | Bug                 | Changes meaning; fix it in the open.      |
| `[tool.pyproject-fmt] column_width = 1` produces ugly output                   | Not a vulnerability | Settings only change layout.              |

## Accepted risks

- Rewriting a file follows a symlink at the named path.
- Formatting is not a safety review: valid but malicious content is reformatted, not removed.

## Keeping this current

Revisit this model when toml-fmt gains a new input source, network access, a new way to run code, or a new place it
writes to, and whenever an advisory is published.
