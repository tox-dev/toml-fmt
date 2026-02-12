<a id="2.16.0"></a>

## 2.16.0 - 2026-02-12

- 🐛 fix(key): normalize literal key quotes to basic (#217) by [@gaborbernat](https://github.com/gaborbernat) in
  [#219](https://github.com/tox-dev/toml-fmt/pull/219)
- Update Rust dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#218](https://github.com/tox-dev/toml-fmt/pull/218) <a id="2.15.3"></a>

## 2.15.3 - 2026-02-11

- ✨ feat(string): add skip_wrap_for_keys config to preserve specific strings by
  [@gaborbernat](https://github.com/gaborbernat) in [#216](https://github.com/tox-dev/toml-fmt/pull/216)
- 🐛 fix(table): normalize quote styles in key sorting by [@gaborbernat](https://github.com/gaborbernat) in
  [#215](https://github.com/tox-dev/toml-fmt/pull/215)
- Update Python dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#210](https://github.com/tox-dev/toml-fmt/pull/210)
- Update Rust dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#209](https://github.com/tox-dev/toml-fmt/pull/209) <a id="2.15.2"></a>

## 2.15.2 - 2026-02-10

- 🐛 fix(array): preserve trailing comments during sort by [@gaborbernat](https://github.com/gaborbernat) in
  [#208](https://github.com/tox-dev/toml-fmt/pull/208) <a id="2.15.1"></a>

## 2.15.1 - 2026-02-10

- 🐛 fix(array): preserve single-line arrays with trailing comments by [@gaborbernat](https://github.com/gaborbernat) in
  [#204](https://github.com/tox-dev/toml-fmt/pull/204)
- 🐛 fix(table): preserve empty tables as inline empty tables by [@gaborbernat](https://github.com/gaborbernat) in
  [#203](https://github.com/tox-dev/toml-fmt/pull/203) <a id="2.15.0"></a>

## 2.15.0 - 2026-02-09

- 🐛 fix(table): preserve comments when collapsing array of tables by [@gaborbernat](https://github.com/gaborbernat) in
  [#198](https://github.com/tox-dev/toml-fmt/pull/198)
- ✨ feat(pyproject-fmt): add tool.coverage key ordering and array sorting by
  [@gaborbernat](https://github.com/gaborbernat) in [#199](https://github.com/tox-dev/toml-fmt/pull/199)
- 🐛 fix(changelog): stop at release commits for orphaned tags by [@gaborbernat](https://github.com/gaborbernat) in
  [#200](https://github.com/tox-dev/toml-fmt/pull/200) <a id="2.14.2"></a>

## 2.14.2 - 2026-02-09

- Release pyproject-fmt 2.14.2 [skip ci] by [@gaborbernat](https://github.com/gaborbernat)
- 🧪 test(coverage): increase coverage to 98% and fix bugs by [@gaborbernat](https://github.com/gaborbernat) in
  [#194](https://github.com/tox-dev/toml-fmt/pull/194)

<a id="2.14.1"></a>

## 2.14.1 - 2026-02-09

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
- ✨ feat(pyproject-fmt): add tool.uv section formatting by [@gaborbernat](https://github.com/gaborbernat) in
  [#169](https://github.com/tox-dev/toml-fmt/pull/169)

<a id="2.14.0"></a>

## 2.14.0 - 2026-02-08

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
- Fix regex for SPDX license normalization by [@alexfikl](https://github.com/alexfikl) in
  [#156](https://github.com/tox-dev/toml-fmt/pull/156)
- Add tests to improve coverage for edge cases by [@gaborbernat](https://github.com/gaborbernat) in
  [#155](https://github.com/tox-dev/toml-fmt/pull/155)

<a id="2.12.1"></a>

## 2.12.1 - 2026-01-31

- Fix expand_tables setting for specific sub-tables by [@gaborbernat](https://github.com/gaborbernat) in
  [#148](https://github.com/tox-dev/toml-fmt/pull/148)
- Use RELEASE_TOKEN to bypass branch protection for releases by [@gaborbernat](https://github.com/gaborbernat) in
  [#147](https://github.com/tox-dev/toml-fmt/pull/147)

<a id="2.12.0"></a>

## 2.12.0 - 2026-01-30

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
- Normalize extra names in optional-dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#135](https://github.com/tox-dev/toml-fmt/pull/135)
- Normalize SPDX operators in license expressions by [@gaborbernat](https://github.com/gaborbernat) in
  [#136](https://github.com/tox-dev/toml-fmt/pull/136)
- Add authors and maintainers sorting by [@gaborbernat](https://github.com/gaborbernat) in
  [#134](https://github.com/tox-dev/toml-fmt/pull/134)
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
- Fix misparsed requires-python with ~= operator (#20) by [@gaborbernat](https://github.com/gaborbernat) in
  [#128](https://github.com/tox-dev/toml-fmt/pull/128)
- Fix build requirements with duplicate package names being removed (#2) by
  [@gaborbernat](https://github.com/gaborbernat) in [#127](https://github.com/tox-dev/toml-fmt/pull/127)
- Improve CI: add Rust coverage thresholds and prek parallel hooks by [@gaborbernat](https://github.com/gaborbernat) in
  [#126](https://github.com/tox-dev/toml-fmt/pull/126)
- Improve GitHub Actions workflows by [@gaborbernat](https://github.com/gaborbernat) in
  [#125](https://github.com/tox-dev/toml-fmt/pull/125)
- Add Nuitka and Typos tool sections by [@kdeldycke](https://github.com/kdeldycke) in
  [#123](https://github.com/tox-dev/toml-fmt/pull/123)

<a id="2.11.1"></a>

## 2.11.1 - 2026-01-07

- Fix parsing of versions in parentheses by [@jamesbursa](https://github.com/jamesbursa) in
  [#122](https://github.com/tox-dev/toml-fmt/pull/122)
- Remove outdated version reference by [@ulgens](https://github.com/ulgens) in
  [#115](https://github.com/tox-dev/toml-fmt/pull/115)
- perf: only compile regexes once by [@henryiii](https://github.com/henryiii) in
  [#110](https://github.com/tox-dev/toml-fmt/pull/110)
- feat: PEP 794 support by [@henryiii](https://github.com/henryiii) in
  [#109](https://github.com/tox-dev/toml-fmt/pull/109)
- Fix tool names in documentation. by [@jorisboeye](https://github.com/jorisboeye) in
  [#106](https://github.com/tox-dev/toml-fmt/pull/106)
- Fix documentation links and bump deps by [@gaborbernat](https://github.com/gaborbernat) in
  [#105](https://github.com/tox-dev/toml-fmt/pull/105)

<a id="2.11.0"></a>

## 2.11.0 - 2025-10-15

- Allow to opt out of the Python version classifier generation by [@gaborbernat](https://github.com/gaborbernat) in
  [#102](https://github.com/tox-dev/toml-fmt/pull/102)
- Fix too aggressive parsing suffix versions by [@gaborbernat](https://github.com/gaborbernat) in
  [#101](https://github.com/tox-dev/toml-fmt/pull/101)

<a id="2.10.0"></a>

## 2.10.0 - 2025-10-09

- Add a few more type checkers to top level tables by [@browniebroke](https://github.com/browniebroke) in
  [#98](https://github.com/tox-dev/toml-fmt/pull/98)

<a id="2.9.0"></a>

## 2.9.0 - 2025-10-08

- Sort [tool.uv] higher in the pyproject.toml by [@browniebroke](https://github.com/browniebroke) in
  [#97](https://github.com/tox-dev/toml-fmt/pull/97)
- Drop 3.9 and add 3.14 by [@gaborbernat](https://github.com/gaborbernat) in
  [#96](https://github.com/tox-dev/toml-fmt/pull/96)

<a id="2.8.0"></a>

## 2.8.0 - 2025-10-08

- Fix parsing of version pre labels by [@adamchainz](https://github.com/adamchainz) in
  [#89](https://github.com/tox-dev/toml-fmt/pull/89)
- Default Python support is now 3.10 to 3.14 by [@gaborbernat](https://github.com/gaborbernat) in
  [#94](https://github.com/tox-dev/toml-fmt/pull/94)

<a id="2.7.0"></a>

## 2.7.0 - 2025-10-01

- Replace upstream pep508 that's deprecated with our own by [@gaborbernat](https://github.com/gaborbernat) in
  [#84](https://github.com/tox-dev/toml-fmt/pull/84)
- Fix empty lines placed within a sorted table by [@ddeepwell](https://github.com/ddeepwell) in
  [#63](https://github.com/tox-dev/toml-fmt/pull/63)
- Do not remove space before .ext by [@hugovk](https://github.com/hugovk) in
  [#76](https://github.com/tox-dev/toml-fmt/pull/76)

<a id="2.6.0"></a>

## 2.6.0 - 2025-05-19

- Use abi3-py39 by [@gaborbernat](https://github.com/gaborbernat) in [#58](https://github.com/tox-dev/toml-fmt/pull/58)
- Bump rust and versions by [@gaborbernat](https://github.com/gaborbernat) in
  [#56](https://github.com/tox-dev/toml-fmt/pull/56)

<a id="2.5.1"></a>

## 2.5.1 - 2025-02-18

- Include all ident parts in keys by [@adamchainz](https://github.com/adamchainz) in
  [#31](https://github.com/tox-dev/toml-fmt/pull/31)
- Add tox-toml-fmt by [@gaborbernat](https://github.com/gaborbernat) in
  [#18](https://github.com/tox-dev/toml-fmt/pull/18)

<a id="2.5.0"></a>

## 2.5.0 - 2024-10-30

- Add support for PEP 735 dependency-groups by [@browniebroke](https://github.com/browniebroke) in
  [#16](https://github.com/tox-dev/toml-fmt/pull/16)
- Extract common Python code to toml-fmt-common by [@gaborbernat](https://github.com/gaborbernat) in
  [#12](https://github.com/tox-dev/toml-fmt/pull/12)
- Fix stray ] in changelog for PR numbers by [@gaborbernat](https://github.com/gaborbernat)

<a id="2.4.3"></a>

## 2.4.3 - 2024-10-17

- Fix tomli not present for Python<3.11 by [@gaborbernat](https://github.com/gaborbernat) in
  [#9](https://github.com/tox-dev/toml-fmt/pull/9)

<a id="2.4.2"></a>

## 2.4.2 - 2024-10-17

- Initial release

<a id="2.4.1"></a>

## 2.4.1 - 2024-10-17

- Initial release

<a id="2.4.0"></a>

## 2.4.0 - 2024-10-17

- Fix GitHub action warning by [@gaborbernat](https://github.com/gaborbernat)
- Create first version by [@gaborbernat](https://github.com/gaborbernat) in
  [#1](https://github.com/tox-dev/toml-fmt/pull/1)

For versions before 2.4.0, see releases in the old repositories:
[pyproject-fmt](https://github.com/tox-dev/pyproject-fmt/releases) (Python) and
[pyproject-fmt-rust](https://github.com/tox-dev/pyproject-fmt-rust/releases) (Rust).
