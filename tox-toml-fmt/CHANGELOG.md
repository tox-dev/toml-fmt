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
