<a id="2.25.3"></a>

## 2.25.3 - 2026-07-13

- 🐛 fix(common): space before marker after URL by [@gaborbernat](https://github.com/gaborbernat) in
  [#410](https://github.com/tox-dev/toml-fmt/pull/410) <a id="2.25.2"></a>

## 2.25.2 - 2026-07-08

- 🐛 fix(common): honor multi-blank table spacing options by [@gaborbernat](https://github.com/gaborbernat) in
  [#403](https://github.com/tox-dev/toml-fmt/pull/403)
- Update Python dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#399](https://github.com/tox-dev/toml-fmt/pull/399)
- 📝 docs: generate formatting examples via the live formatter by [@gaborbernat](https://github.com/gaborbernat) in
  [#397](https://github.com/tox-dev/toml-fmt/pull/397) <a id="2.25.1"></a>

## 2.25.1 - 2026-06-25

- 📝 docs: keep comments to why only and de-slop docs by [@gaborbernat](https://github.com/gaborbernat) in
  [#394](https://github.com/tox-dev/toml-fmt/pull/394)
- 🐛 fix(common): reorder and re-comment disabled keys safely by [@gaborbernat](https://github.com/gaborbernat) in
  [#391](https://github.com/tox-dev/toml-fmt/pull/391)
- 🐛 fix(common): preserve array entry trivia when reordering inline tables by
  [@gaborbernat](https://github.com/gaborbernat) in [#392](https://github.com/tox-dev/toml-fmt/pull/392)
- 🐛 fix(common): decode line-ending backslash in multiline basic strings by
  [@gaborbernat](https://github.com/gaborbernat) in [#393](https://github.com/tox-dev/toml-fmt/pull/393)
- fix: Sort `base_python_file` key in tox environments by [@edgarrmondragon](https://github.com/edgarrmondragon) in
  [#386](https://github.com/tox-dev/toml-fmt/pull/386) <a id="2.25.0"></a>

## 2.25.0 - 2026-06-17

- 🐛 fix(common): keep empty table when sub-table stays expanded by [@gaborbernat](https://github.com/gaborbernat) in
  [#385](https://github.com/tox-dev/toml-fmt/pull/385)
- ✨ feat(common): share disabled-key handling across formatters by [@gaborbernat](https://github.com/gaborbernat) in
  [#383](https://github.com/tox-dev/toml-fmt/pull/383)
- uv direct tool install support by [@bn-andrew](https://github.com/bn-andrew) in
  [#382](https://github.com/tox-dev/toml-fmt/pull/382)
- ✨ feat(common): keep disabled keys anchored to their table by [@gaborbernat](https://github.com/gaborbernat) in
  [#381](https://github.com/tox-dev/toml-fmt/pull/381)
- ✨ feat(common): add # Group: markers for scoped reordering by [@gaborbernat](https://github.com/gaborbernat) in
  [#380](https://github.com/tox-dev/toml-fmt/pull/380)
- Update Python dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#377](https://github.com/tox-dev/toml-fmt/pull/377)
- Update Rust dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#378](https://github.com/tox-dev/toml-fmt/pull/378) <a id="2.24.0"></a>

## 2.24.0 - 2026-06-11

- ✨ feat(pyproject-fmt): drop redundant wheel from setuptools build requires by
  [@gaborbernat](https://github.com/gaborbernat) in [#373](https://github.com/tox-dev/toml-fmt/pull/373)
- Update Python dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#368](https://github.com/tox-dev/toml-fmt/pull/368) <a id="2.23.0"></a>

## 2.23.0 - 2026-05-30

- 📝 docs(pyproject-fmt): overhaul the formatting reference by [@gaborbernat](https://github.com/gaborbernat) in
  [#366](https://github.com/tox-dev/toml-fmt/pull/366) <a id="2.22.0"></a>

## 2.22.0 - 2026-05-29

- ✨ feat(build): ship self-contained pyproject-fmt and tox-toml-fmt wheels by
  [@gaborbernat](https://github.com/gaborbernat) in [#363](https://github.com/tox-dev/toml-fmt/pull/363)
- 🐛 fix(common): restore \_build_cli alias for backward compatibility by [@gaborbernat](https://github.com/gaborbernat)
  in [#361](https://github.com/tox-dev/toml-fmt/pull/361)
- ✨ feat(pyproject-fmt): add [tool.ty] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#354](https://github.com/tox-dev/toml-fmt/pull/354)
- ✨ feat(pyproject-fmt): add [tool.deptry] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#353](https://github.com/tox-dev/toml-fmt/pull/353)
- ✨ feat(pyproject-fmt): add [tool.autopep8] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#352](https://github.com/tox-dev/toml-fmt/pull/352)
- ✨ feat(pyproject-fmt): add [tool.vulture] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#351](https://github.com/tox-dev/toml-fmt/pull/351)
- ✨ feat(pyproject-fmt): add [tool.docformatter] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#350](https://github.com/tox-dev/toml-fmt/pull/350)
- ✨ feat(pyproject-fmt): add [tool.interrogate] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#349](https://github.com/tox-dev/toml-fmt/pull/349)
- ✨ feat(pyproject-fmt): add [tool.bumpversion] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#348](https://github.com/tox-dev/toml-fmt/pull/348)
- ✨ feat(pyproject-fmt): add [tool.scikit-build] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#347](https://github.com/tox-dev/toml-fmt/pull/347)
- ✨ feat(pyproject-fmt): add [tool.semantic_release] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#346](https://github.com/tox-dev/toml-fmt/pull/346)
- ✨ feat(pyproject-fmt): add [tool.pyrefly] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#345](https://github.com/tox-dev/toml-fmt/pull/345)
- ✨ feat(pyproject-fmt): add [tool.check-manifest] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#344](https://github.com/tox-dev/toml-fmt/pull/344)
- ✨ feat(pyproject-fmt): add [tool.yapf] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#343](https://github.com/tox-dev/toml-fmt/pull/343)
- ✨ feat(pyproject-fmt): add [tool.djlint] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#342](https://github.com/tox-dev/toml-fmt/pull/342)
- ✨ feat(pyproject-fmt): add [tool.pylint.\*] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#341](https://github.com/tox-dev/toml-fmt/pull/341)
- ✨ feat(pyproject-fmt): add [tool.towncrier] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#340](https://github.com/tox-dev/toml-fmt/pull/340)
- ✨ feat(pyproject-fmt): add [tool.codespell] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#338](https://github.com/tox-dev/toml-fmt/pull/338)
- ✨ feat(pyproject-fmt): add [tool.maturin] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#337](https://github.com/tox-dev/toml-fmt/pull/337)
- ✨ feat(pyproject-fmt): add [tool.bandit] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#336](https://github.com/tox-dev/toml-fmt/pull/336)
- ✨ feat(pyproject-fmt): add [tool.tox] handler reusing tox-toml-fmt rules by
  [@gaborbernat](https://github.com/gaborbernat) in [#335](https://github.com/tox-dev/toml-fmt/pull/335)
- ✨ feat(pyproject-fmt): add [tool.cibuildwheel] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#334](https://github.com/tox-dev/toml-fmt/pull/334)
- ✨ feat(pyproject-fmt): add [tool.pdm.\*] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#333](https://github.com/tox-dev/toml-fmt/pull/333)
- ✨ feat(pyproject-fmt): add [tool.pyright] + [tool.basedpyright] handler by
  [@gaborbernat](https://github.com/gaborbernat) in [#332](https://github.com/tox-dev/toml-fmt/pull/332)
- ✨ feat(pyproject-fmt): add [tool.isort] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#331](https://github.com/tox-dev/toml-fmt/pull/331)
- ✨ feat(pyproject-fmt): add [tool.hatch.\*] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#329](https://github.com/tox-dev/toml-fmt/pull/329)
- ✨ feat(pyproject-fmt): add [tool.black] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#328](https://github.com/tox-dev/toml-fmt/pull/328)
- ✨ feat(pyproject-fmt): add [tool.pytest.ini_options] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#327](https://github.com/tox-dev/toml-fmt/pull/327)
- ✨ feat(pyproject-fmt): add [tool.setuptools] + [tool.setuptools_scm] handlers by
  [@gaborbernat](https://github.com/gaborbernat) in [#326](https://github.com/tox-dev/toml-fmt/pull/326)
- ✨ feat(pyproject-fmt): add [tool.mypy] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#325](https://github.com/tox-dev/toml-fmt/pull/325)
- ✨ feat(pyproject-fmt): add [tool.poetry] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#324](https://github.com/tox-dev/toml-fmt/pull/324)
- ✨ feat(pyproject-fmt): add [tool.commitizen] handler by [@gaborbernat](https://github.com/gaborbernat) in
  [#339](https://github.com/tox-dev/toml-fmt/pull/339)
- Update Python dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#358](https://github.com/tox-dev/toml-fmt/pull/358)
- 🐛 fix(common): preserve triple-literal strings when re-emitting by [@gaborbernat](https://github.com/gaborbernat) in
  [#356](https://github.com/tox-dev/toml-fmt/pull/356)
- ♻️ refactor(common): deduplicate table formatting CLI args by [@gaborbernat](https://github.com/gaborbernat) in
  [#320](https://github.com/tox-dev/toml-fmt/pull/320)
- ✨ feat(common): add configurable table spacing options by [@gaborbernat](https://github.com/gaborbernat) in
  [#319](https://github.com/tox-dev/toml-fmt/pull/319)
- Update Python dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#310](https://github.com/tox-dev/toml-fmt/pull/310) <a id="2.21.2"></a>

## 2.21.2 - 2026-05-05

- ✨ feat(build): support free-threaded Python wheels by [@gaborbernat](https://github.com/gaborbernat) in
  [#307](https://github.com/tox-dev/toml-fmt/pull/307)
- 🐛 fix(common): skip empty tables in Tables::get by [@gaborbernat](https://github.com/gaborbernat) in
  [#304](https://github.com/tox-dev/toml-fmt/pull/304)
- Update Python dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#301](https://github.com/tox-dev/toml-fmt/pull/301)
- Update Python dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#295](https://github.com/tox-dev/toml-fmt/pull/295)
- Update Rust dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#296](https://github.com/tox-dev/toml-fmt/pull/296) <a id="2.21.1"></a>

## 2.21.1 - 2026-04-13

- 🐛 fix(pyproject-fmt): produce valid TOML when sorting arrays with value on bracket line by
  [@gaborbernat](https://github.com/gaborbernat) in [#293](https://github.com/tox-dev/toml-fmt/pull/293)
- Update Rust dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#287](https://github.com/tox-dev/toml-fmt/pull/287)
- Update Python dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#286](https://github.com/tox-dev/toml-fmt/pull/286) <a id="2.20.0"></a>

## 2.20.0 - 2026-03-18

- ✨ feat(pyproject-fmt): add bandit to recognized linters by [@gaborbernat](https://github.com/gaborbernat) in
  [#276](https://github.com/tox-dev/toml-fmt/pull/276)
- Update Python dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#269](https://github.com/tox-dev/toml-fmt/pull/269) <a id="2.19.0"></a>

## 2.19.0 - 2026-03-16

- ♻️ refactor(pyproject-fmt): sort type checkers after linters by [@gaborbernat](https://github.com/gaborbernat) in
  [#274](https://github.com/tox-dev/toml-fmt/pull/274)
- Update Rust dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#270](https://github.com/tox-dev/toml-fmt/pull/270) <a id="2.18.1"></a>

## 2.18.1 - 2026-03-03

- 🐛 fix(common): panic on non-array nodes in array ops by [@gaborbernat](https://github.com/gaborbernat) in
  [#266](https://github.com/tox-dev/toml-fmt/pull/266) <a id="2.18.0"></a>

## 2.18.0 - 2026-03-03

- ✨ feat(tox-toml-fmt): add inline table key reordering by [@gaborbernat](https://github.com/gaborbernat) in
  [#264](https://github.com/tox-dev/toml-fmt/pull/264) <a id="2.17.0"></a>

## 2.17.0 - 2026-03-01

- ✨ feat(common): add shared config file support by [@gaborbernat](https://github.com/gaborbernat) in
  [#258](https://github.com/tox-dev/toml-fmt/pull/258)
- 🐛 fix(parser): adapt to tombi v0.8.0 AST changes by [@gaborbernat](https://github.com/gaborbernat) in
  [#259](https://github.com/tox-dev/toml-fmt/pull/259)
- Update Python dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#257](https://github.com/tox-dev/toml-fmt/pull/257)
- Update Python dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#248](https://github.com/tox-dev/toml-fmt/pull/248)
- Update Python dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#247](https://github.com/tox-dev/toml-fmt/pull/247)
- Update Python dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#245](https://github.com/tox-dev/toml-fmt/pull/245) <a id="2.16.2"></a>

## 2.16.2 - 2026-02-23

- ⬆️ build(deps): update Rust and Python dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#243](https://github.com/tox-dev/toml-fmt/pull/243)
- Fix panic on non-valid classifiers by [@Nicolaus93](https://github.com/Nicolaus93) in
  [#241](https://github.com/tox-dev/toml-fmt/pull/241)
- Update Python dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#240](https://github.com/tox-dev/toml-fmt/pull/240)
- Update Python dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#238](https://github.com/tox-dev/toml-fmt/pull/238)
- Update Python dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#236](https://github.com/tox-dev/toml-fmt/pull/236)
- Update Rust dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#235](https://github.com/tox-dev/toml-fmt/pull/235)
- 🐛 fix(tox-toml-fmt): handle quoted keys with dots in env tables by [@gaborbernat](https://github.com/gaborbernat) in
  [#234](https://github.com/tox-dev/toml-fmt/pull/234)
- Update Python dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#232](https://github.com/tox-dev/toml-fmt/pull/232)
- ✨ feat(tox-toml-fmt): add semantic formatting matching tox-ini-fmt by [@gaborbernat](https://github.com/gaborbernat)
  in [#230](https://github.com/tox-dev/toml-fmt/pull/230) <a id="2.16.1"></a>

## 2.16.1 - 2026-02-18

- 🐛 fix(project): stop sorting authors and maintainers (#228) by [@gaborbernat](https://github.com/gaborbernat) in
  [#229](https://github.com/tox-dev/toml-fmt/pull/229)
- Update Python dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#227](https://github.com/tox-dev/toml-fmt/pull/227)
- Update Rust dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#226](https://github.com/tox-dev/toml-fmt/pull/226)
- Update Python dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#225](https://github.com/tox-dev/toml-fmt/pull/225)
- Update Python dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#224](https://github.com/tox-dev/toml-fmt/pull/224)
- 📝🐛 docs(config): document column_width string wrapping by [@gaborbernat](https://github.com/gaborbernat) in
  [#223](https://github.com/tox-dev/toml-fmt/pull/223)
- Update Python dependencies by [@gaborbernat](https://github.com/gaborbernat) in
  [#221](https://github.com/tox-dev/toml-fmt/pull/221) <a id="2.16.0"></a>

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
- Collapse \[[project.authors]\] array of tables to inline format by [@gaborbernat](https://github.com/gaborbernat) in
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
- Fix stray \] in changelog for PR numbers by [@gaborbernat](https://github.com/gaborbernat)

<a id="2.4.3"></a>

## 2.4.3 - 2024-10-17

- Fix tomli not present for Python\<3.11 by [@gaborbernat](https://github.com/gaborbernat) in
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
