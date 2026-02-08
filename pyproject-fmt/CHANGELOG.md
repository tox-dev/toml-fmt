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
