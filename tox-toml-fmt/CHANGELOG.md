<a id="1.5.2"></a>

## 1.5.2 - 2026-02-09

- Release tox-toml-fmt 1.5.2 [skip ci] by [@gaborbernat](https://github.com/gaborbernat)
- 🧪 test(coverage): increase coverage to 98% and fix bugs by [@gaborbernat](https://github.com/gaborbernat) in
  [#194](https://github.com/tox-dev/toml-fmt/pull/194)

<a id="1.5.1"></a>

## 1.5.1 - 2026-02-08

- 🐛 fix(table): preserve table headers and reduce duplication by [@gaborbernat](https://github.com/gaborbernat) in
  [#192](https://github.com/tox-dev/toml-fmt/pull/192)
- 🐛 fix(array): preserve comments with strings by [@gaborbernat](https://github.com/gaborbernat) in
  [#191](https://github.com/tox-dev/toml-fmt/pull/191)
- 🧪 test: add idempotency tests for string wrapping by [@gaborbernat](https://github.com/gaborbernat) in
  [#190](https://github.com/tox-dev/toml-fmt/pull/190)
- 🐛 fix(parser): handle comments with double quotes in arrays by [@gaborbernat](https://github.com/gaborbernat) in
  [#189](https://github.com/tox-dev/toml-fmt/pull/189)
- 🐛 fix(array): sort by value not comment for entries with leading comments by
  [@gaborbernat](https://github.com/gaborbernat) in [#188](https://github.com/tox-dev/toml-fmt/pull/188)
- Update Rust dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#183](https://github.com/tox-dev/toml-fmt/pull/183)
- Update Python dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#178](https://github.com/tox-dev/toml-fmt/pull/178)
- ✨ feat(workspace): internalize toml-fmt-common as workspace member by [@gaborbernat](https://github.com/gaborbernat)
  in [#170](https://github.com/tox-dev/toml-fmt/pull/170)

<a id="1.5.0"></a>

## 1.5.0 - 2026-02-08

- ✨ feat(changelog): add --regenerate flag for full changelog rebuild by [@gaborbernat](https://github.com/gaborbernat)
  in [#166](https://github.com/tox-dev/toml-fmt/pull/166)
- 🐛 fix(readme): use explicit UTF-8 encoding for file operations by [@gaborbernat](https://github.com/gaborbernat) in
  [#165](https://github.com/tox-dev/toml-fmt/pull/165)
- 📝 docs(formatting): restructure docs and fix array formatting behavior by
  [@gaborbernat](https://github.com/gaborbernat) in [#164](https://github.com/tox-dev/toml-fmt/pull/164)
- ♻️ refactor(parser): migrate from taplo to tombi by [@gaborbernat](https://github.com/gaborbernat) in
  [#159](https://github.com/tox-dev/toml-fmt/pull/159)
- Fix expand_tables setting for deeply nested tables (#146) by [@gaborbernat](https://github.com/gaborbernat) in
  [#160](https://github.com/tox-dev/toml-fmt/pull/160)
- Prefer double quotes, use single quotes to avoid escaping by [@gaborbernat](https://github.com/gaborbernat) in
  [#162](https://github.com/tox-dev/toml-fmt/pull/162)
- Fix comment before table moving incorrectly when table has comments by [@gaborbernat](https://github.com/gaborbernat)
  in [#158](https://github.com/tox-dev/toml-fmt/pull/158)
- Add tests to improve coverage for edge cases by [@gaborbernat](https://github.com/gaborbernat) in
  [#155](https://github.com/tox-dev/toml-fmt/pull/155)
- Fix expand_tables setting for specific sub-tables by [@gaborbernat](https://github.com/gaborbernat) in
  [#148](https://github.com/tox-dev/toml-fmt/pull/148)
- Use RELEASE_TOKEN to bypass branch protection for releases by [@gaborbernat](https://github.com/gaborbernat) in
  [#147](https://github.com/tox-dev/toml-fmt/pull/147)

<a id="1.3.0"></a>

## 1.3.0 - 2026-01-30

- Generate README.rst dynamically from docs at build time by [@gaborbernat](https://github.com/gaborbernat) in
  [#145](https://github.com/tox-dev/toml-fmt/pull/145)
- 📚 Document formatting principles and normalizations by [@gaborbernat](https://github.com/gaborbernat) in
  [#144](https://github.com/tox-dev/toml-fmt/pull/144)
- Improve maintainalibility by [@gaborbernat](https://github.com/gaborbernat) in
  [#143](https://github.com/tox-dev/toml-fmt/pull/143)
- Add configurable table formatting to pyproject-fmt and order tox env tables by env_list by
  [@gaborbernat](https://github.com/gaborbernat) in [#142](https://github.com/tox-dev/toml-fmt/pull/142)
- Order tox env tables according to env_list and add codecov token by [@gaborbernat](https://github.com/gaborbernat) in
  [#141](https://github.com/tox-dev/toml-fmt/pull/141)
- Fix comments before table headers staying with correct table by [@gaborbernat](https://github.com/gaborbernat) in
  [#140](https://github.com/tox-dev/toml-fmt/pull/140)
- Sort subtables alphabetically within the same tool by [@gaborbernat](https://github.com/gaborbernat) in
  [#139](https://github.com/tox-dev/toml-fmt/pull/139)
- Collapse [[project.authors]] array of tables to inline format by [@gaborbernat](https://github.com/gaborbernat) in
  [#137](https://github.com/tox-dev/toml-fmt/pull/137)
- Bump toml-fmt-common to 1.2 by [@gaborbernat](https://github.com/gaborbernat) in
  [#138](https://github.com/tox-dev/toml-fmt/pull/138)
- Add keyword and classifier deduplication by [@gaborbernat](https://github.com/gaborbernat) in
  [#133](https://github.com/tox-dev/toml-fmt/pull/133)
- Fix crash on multi-line strings with line continuation by [@gaborbernat](https://github.com/gaborbernat) in
  [#132](https://github.com/tox-dev/toml-fmt/pull/132)
- Add PEP 794 private dependency support by [@gaborbernat](https://github.com/gaborbernat) in
  [#131](https://github.com/tox-dev/toml-fmt/pull/131)
- Fix literal strings with invalid escapes being corrupted by [@gaborbernat](https://github.com/gaborbernat) in
  [#130](https://github.com/tox-dev/toml-fmt/pull/130)
- Remove testpaths config to fix sdist warning (#120) by [@gaborbernat](https://github.com/gaborbernat) in
  [#129](https://github.com/tox-dev/toml-fmt/pull/129)
- Fix build requirements with duplicate package names being removed (#2) by
  [@gaborbernat](https://github.com/gaborbernat) in [#127](https://github.com/tox-dev/toml-fmt/pull/127)
- Improve CI: add Rust coverage thresholds and prek parallel hooks by [@gaborbernat](https://github.com/gaborbernat) in
  [#126](https://github.com/tox-dev/toml-fmt/pull/126)
- Improve GitHub Actions workflows by [@gaborbernat](https://github.com/gaborbernat) in
  [#125](https://github.com/tox-dev/toml-fmt/pull/125)

<a id="1.2.2"></a>

## 1.2.2 - 2026-01-07

- Fix parsing of versions in parentheses by [@jamesbursa](https://github.com/jamesbursa) in
  [#122](https://github.com/tox-dev/toml-fmt/pull/122)

<a id="1.2.1"></a>

## 1.2.1 - 2025-11-12

- Fix tool names in documentation. by [@jorisboeye](https://github.com/jorisboeye) in
  [#106](https://github.com/tox-dev/toml-fmt/pull/106)
- Fix documentation links and bump deps by [@gaborbernat](https://github.com/gaborbernat) in
  [#105](https://github.com/tox-dev/toml-fmt/pull/105)
- Fix too aggressive parsing suffix versions by [@gaborbernat](https://github.com/gaborbernat) in
  [#101](https://github.com/tox-dev/toml-fmt/pull/101)

<a id="1.2.0"></a>

## 1.2.0 - 2025-10-08

- Drop 3.9 and add 3.14 by [@gaborbernat](https://github.com/gaborbernat) in
  [#96](https://github.com/tox-dev/toml-fmt/pull/96)
- Fix parsing of version pre labels by [@adamchainz](https://github.com/adamchainz) in
  [#89](https://github.com/tox-dev/toml-fmt/pull/89)

<a id="1.1.0"></a>

## 1.1.0 - 2025-10-01

- Replace upstream pep508 that's deprecated with our own by [@gaborbernat](https://github.com/gaborbernat) in
  [#84](https://github.com/tox-dev/toml-fmt/pull/84)
- Fix empty lines placed within a sorted table by [@ddeepwell](https://github.com/ddeepwell) in
  [#63](https://github.com/tox-dev/toml-fmt/pull/63)
- Use abi3-py39 by [@gaborbernat](https://github.com/gaborbernat) in [#58](https://github.com/tox-dev/toml-fmt/pull/58)
- Bump rust and versions by [@gaborbernat](https://github.com/gaborbernat) in
  [#56](https://github.com/tox-dev/toml-fmt/pull/56)
- Include all ident parts in keys by [@adamchainz](https://github.com/adamchainz) in
  [#31](https://github.com/tox-dev/toml-fmt/pull/31)

<a id="1.0.0"></a>

## 1.0.0 - 2024-10-31

- Update Cargo.toml by [@gaborbernat](https://github.com/gaborbernat)
- Add tox-toml-fmt by [@gaborbernat](https://github.com/gaborbernat) in
  [#18](https://github.com/tox-dev/toml-fmt/pull/18)
- Add support for PEP 735 dependency-groups by [@browniebroke](https://github.com/browniebroke) in
  [#16](https://github.com/tox-dev/toml-fmt/pull/16)
- Extract common Python code to toml-fmt-common by [@gaborbernat](https://github.com/gaborbernat) in
  [#12](https://github.com/tox-dev/toml-fmt/pull/12)
- Create first version by [@gaborbernat](https://github.com/gaborbernat) in
  [#1](https://github.com/tox-dev/toml-fmt/pull/1)
