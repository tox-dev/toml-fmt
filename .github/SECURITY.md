# Security policy

toml-fmt builds pyproject-fmt and tox-toml-fmt, comment-preserving formatters for `pyproject.toml` and `tox.toml` that
share a Rust core. This policy says how to report a security problem and how we decide whether a report is one.

## Reporting a vulnerability

**Do not open a public issue for a security problem.** Use
[GitHub's private vulnerability reporting](https://github.com/tox-dev/toml-fmt/security/advisories/new). If you do not
have a GitHub account, or you are a Tidelift subscriber, report through the
[Tidelift security contact](https://tidelift.com/security) instead; Tidelift coordinates the fix and the disclosure with
us. If neither works for you, email [gaborjbernat@gmail.com](mailto:gaborjbernat@gmail.com).

Include what the attacker controls and what they gain, the steps or a proof of concept, and the versions and platform
you tested. Write the report yourself; unreviewed tool or model output takes longer to triage.

## What to expect

Volunteers maintain toml-fmt, with no on-call rotation. We aim to acknowledge a report within **5 working days** and
assess it within **15 working days**. Fixes are developed in a private fork attached to the draft advisory, ship in a
new release, and the advisory, with a CVE requested through GitHub, goes out with that release. The default embargo is
**90 days** from the report; we agree any change with the reporter. We credit reporters in the advisory under the name
they choose, or not at all. There is no bug bounty.

## Supported versions

Only the latest release receives fixes. Confirm the problem reproduces on it before reporting.

## Vulnerability or bug

A report is a vulnerability when an attacker who controls only an input that the [threat model](THREAT_MODEL.md) lists
as untrusted breaks a guarantee toml-fmt makes. A report that first needs control of something the threat model trusts,
such as the command-line options, the configuration or the files you choose to format, is a bug: we fix it in the open
and publish no advisory. Classify by who supplies the dangerous input, not by how bad the outcome sounds.

**In scope**

- Formatting an untrusted TOML file, for example one from a pull request in CI, executing code, reading or writing any
  file other than the one named, or making a network request.
- A small input that makes the formatter hang or exhaust memory.

**Out of scope**

- Output that changes what a file means, or drops content: a correctness bug we want reported in the open.
- A named path that is a symlink: the formatter reads and rewrites its target, as the caller asked.
- Malformed input that fails with an error.

If you cannot tell which side a finding falls on, report it privately and we will decide.
