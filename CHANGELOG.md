# Changelog

## pyproject-fmt

### 2.30.4 (2026-10-27)

<a name="release-pyproject-fmt-2-30-4"></a>

<a name="release-pyproject-fmt-2-30-4-packaging"></a>

#### Packaging

- Releases now run on the shared tox-dev/github-workflows pipeline, with a locked release toolchain and generated GitHub
  Release notes ([#494](https://github.com/tox-dev/toml-fmt/pull/494)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 2.30.3 (2026-10-10)

<a name="release-pyproject-fmt-2-30-3"></a>

<a name="release-pyproject-fmt-2-30-3-added"></a>

#### Added

- **Require 3.11 and default to 3.15.** ([#480](https://github.com/tox-dev/toml-fmt/pull/480)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 2.30.2 (2026-10-09)

<a name="release-pyproject-fmt-2-30-2"></a>

<a name="release-pyproject-fmt-2-30-2-fixed"></a>

#### Fixed

- **Format every input after an earlier one changes.** ([#479](https://github.com/tox-dev/toml-fmt/pull/479)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 2.30.1 (2026-10-04)

<a name="release-pyproject-fmt-2-30-1"></a>

<a name="release-pyproject-fmt-2-30-1-added"></a>

#### Added

- **Raise the default minimum Python to 3.11.** ([#476](https://github.com/tox-dev/toml-fmt/pull/476)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 2.30.0 (2026-10-02)

<a name="release-pyproject-fmt-2-30-0"></a>

### 2.29.4 (2026-09-08)

<a name="release-pyproject-fmt-2-29-4"></a>

<a name="release-pyproject-fmt-2-29-4-fixed"></a>

#### Fixed

- **Keep the classifiers a bound admits.** ([#463](https://github.com/tox-dev/toml-fmt/pull/463)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 2.29.3 (2026-09-01)

<a name="release-pyproject-fmt-2-29-3"></a>

<a name="release-pyproject-fmt-2-29-3-fixed"></a>

#### Fixed

- **The test suite the source distribution ships passes from the unpacked sdist.** The backend read the project metadata
  from beside itself, and the sdist keeps a copy of that file next to the tests, a directory below the `pyproject.toml`
  it belongs to, so eight tests ended on the file it could not find. It reads the metadata from either place.
  ([#454](https://github.com/tox-dev/toml-fmt/issues/454), [#457](https://github.com/tox-dev/toml-fmt/pull/457)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 2.29.2 (2026-08-31)

<a name="release-pyproject-fmt-2-29-2"></a>

<a name="release-pyproject-fmt-2-29-2-fixed"></a>

#### Fixed

- **A build from the source distribution runs on what that distribution carries.** 2.29.0 named toml-fmt-common as a
  dependency and resolved it from PyPI, where the newest release dates from May; 2.29.1 dropped the dependency before
  the sdist carried the sources, so a build from it failed on the import. This release carries them, and its metadata
  requires nothing. ([#450](https://github.com/tox-dev/toml-fmt/issues/450),
  [#452](https://github.com/tox-dev/toml-fmt/pull/452)) by [@gaborbernat](https://github.com/gaborbernat).

  The wheels held their vendored copy throughout, so an install that takes a wheel was never affected.

### 2.29.1 (2026-08-31)

<a name="release-pyproject-fmt-2-29-1"></a>

<a name="release-pyproject-fmt-2-29-1-fixed"></a>

#### Fixed

- **The source distribution no longer names toml-fmt-common as a dependency.** The newest release of that package on
  PyPI dates from May, months behind this one, so a build from the sdist ran an old settings reader against the current
  formatter and failed a large part of its own test suite. ([#450](https://github.com/tox-dev/toml-fmt/issues/450),
  [#451](https://github.com/tox-dev/toml-fmt/pull/451)) by [@gaborbernat](https://github.com/gaborbernat).

<a name="release-pyproject-fmt-2-29-1-packaging"></a>

#### Packaging

- **This release's source distribution carries no toml-fmt-common, so a build from it fails on the import.**
  [2.29.2](https://github.com/tox-dev/toml-fmt/releases/tag/pyproject-fmt/2.29.2) carries it. The wheels hold their
  vendored copy and install as before. ([#452](https://github.com/tox-dev/toml-fmt/pull/452)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 2.29.0 (2026-08-31)

<a name="release-pyproject-fmt-2-29-0"></a>

<a name="release-pyproject-fmt-2-29-0-changes"></a>

#### Changes

- **A string measures from the start of its key.** A long key can be what pushes a value past `column_width`; measuring
  the value alone left lines running past the column the setting asks for.
  ([#448](https://github.com/tox-dev/toml-fmt/pull/448)) by [@gaborbernat](https://github.com/gaborbernat).
- **Deep input comes back as an error rather than a crash.** Reading a value, writing it and dropping it each walk it by
  calling themselves, so the model caps nesting at 256 levels. A 12,000-deep value used to end the process.
  ([#448](https://github.com/tox-dev/toml-fmt/pull/448)) by [@gaborbernat](https://github.com/gaborbernat).

<a name="release-pyproject-fmt-2-29-0-fixed"></a>

#### Fixed

- **A requirement written with a space after its operator reads as a requirement.** `requires-python = ">= 3.12"` did
  not parse as a version bound, so the generated classifiers fell back to the configured floor and ceiling. `pypa/build`
  carried a 3.9 classifier it does not support. ([#448](https://github.com/tox-dev/toml-fmt/pull/448)) by
  [@gaborbernat](https://github.com/gaborbernat).
- **The classifier window works at patch precision.** `requires-python = "<3.10.1"` lost the whole 3.10 series, and
  claimed `Programming Language :: Python :: 3 :: Only` for a bound that admits Python 2.
  ([#448](https://github.com/tox-dev/toml-fmt/pull/448)) by [@gaborbernat](https://github.com/gaborbernat).
- **A literal string sorts with the values around it.** `'zz'` and `'aa'` held their order while the same values in
  double quotes sorted. ([#448](https://github.com/tox-dev/toml-fmt/pull/448)) by
  [@gaborbernat](https://github.com/gaborbernat).
- **A comment written before a member's comma stays on that member's line.** The comma is what says which member a
  comment belongs to: one written before it closes that member's line, one written after it leads the next member. Both
  used to end up on a line of their own, leaving the comma stranded below.
  ([#448](https://github.com/tox-dev/toml-fmt/pull/448)) by [@gaborbernat](https://github.com/gaborbernat).
- **`classifiers` written as a string stays as written.** Asking for generated classifiers replaced the string with an
  array, losing what the file said. ([#448](https://github.com/tox-dev/toml-fmt/pull/448)) by
  [@gaborbernat](https://github.com/gaborbernat).
- **Folding sub-tables into their parent no longer depends on the order the file wrote them in.**
  ([#448](https://github.com/tox-dev/toml-fmt/pull/448)) by [@gaborbernat](https://github.com/gaborbernat).
- **The key order covers pyrefly's documented option names.** It spells them with hyphens, the order listed only the
  underscore forms, and a file using the documented spelling fell through to alphabetical order.
  ([#448](https://github.com/tox-dev/toml-fmt/pull/448)) by [@gaborbernat](https://github.com/gaborbernat).
- **A dependency group keeps its `include-group` entries where the file wrote them.** An `include-group` pulls its group
  in at the point it sits, so moving it changes what the group resolves to.
  ([#448](https://github.com/tox-dev/toml-fmt/pull/448)) by [@gaborbernat](https://github.com/gaborbernat).
- **Free-form license text stays as written.** `license = "MIT or later"` came back rewritten as though it were an SPDX
  expression; the formatter now rewrites the value only once it parses as one over registered identifiers.
  ([#448](https://github.com/tox-dev/toml-fmt/pull/448)) by [@gaborbernat](https://github.com/gaborbernat).
- **The `*` catch-all in a setuptools data table matches the way the file spells it.** `*` is not a name TOML reads
  bare, so a file writes it quoted; the catch-all led the table only because a quote happens to sort before a letter.
  ([#448](https://github.com/tox-dev/toml-fmt/pull/448)) by [@gaborbernat](https://github.com/gaborbernat).

<a name="release-pyproject-fmt-2-29-0-performance"></a>

#### Performance

- **The quadratic walks the old parse tree forced are gone.** 32,000 interleaved root keys under two tables took 618
  seconds and now take 4.8; scaling is close to linear. ([#448](https://github.com/tox-dev/toml-fmt/pull/448)) by
  [@gaborbernat](https://github.com/gaborbernat).

<a name="release-pyproject-fmt-2-29-0-packaging"></a>

#### Packaging

- **This release's source distribution names a dependency on toml-fmt-common without carrying it, and the newest release
  of that package on PyPI dates from May.** A build from the sdist therefore runs an older settings reader against this
  formatter. [2.29.2](https://github.com/tox-dev/toml-fmt/releases/tag/pyproject-fmt/2.29.2) carries the sources. The
  wheels hold their vendored copy and install as before. ([#450](https://github.com/tox-dev/toml-fmt/issues/450))

### 2.28.2 (2026-08-28)

<a name="release-pyproject-fmt-2-28-2"></a>

<a name="release-pyproject-fmt-2-28-2-fixed"></a>

#### Fixed

- **A file keeps the line endings it was written with.** A CRLF file came back LF, turning every line of a Windows
  checkout into a diff. ([#447](https://github.com/tox-dev/toml-fmt/pull/447)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 2.28.1 (2026-08-24)

<a name="release-pyproject-fmt-2-28-1"></a>

<a name="release-pyproject-fmt-2-28-1-packaging"></a>

#### Packaging

- **The wheel declares its dependencies in the METADATA header**, where an installer looks for them.
  ([#444](https://github.com/tox-dev/toml-fmt/pull/444)) by [@gaborbernat](https://github.com/gaborbernat).

### 2.28.0 (2026-08-16)

<a name="release-pyproject-fmt-2-28-0"></a>

<a name="release-pyproject-fmt-2-28-0-fixed"></a>

#### Fixed

- **A sub-table written under its own header reads in the same key order as the folded form.**
  ([#438](https://github.com/tox-dev/toml-fmt/pull/438)) by [@gaborbernat](https://github.com/gaborbernat).
- **`[tool.cibuildwheel]` overrides keep their order once folded.**
  ([#437](https://github.com/tox-dev/toml-fmt/pull/437)) by [@gaborbernat](https://github.com/gaborbernat).
- **`[tool.coverage]` keys follow the order the tool documents.** ([#436](https://github.com/tox-dev/toml-fmt/pull/436))
  by [@gaborbernat](https://github.com/gaborbernat).

<a name="release-pyproject-fmt-2-28-0-documentation"></a>

#### Documentation

- **The docs build publishes `llms.txt`.** ([#432](https://github.com/tox-dev/toml-fmt/pull/432)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 2.27.1 (2026-08-12)

<a name="release-pyproject-fmt-2-27-1"></a>

<a name="release-pyproject-fmt-2-27-1-fixed"></a>

#### Fixed

- **`project.version` stays as written.** It is the exact version published in the package metadata, and normalizing it
  rewrote what a release had already gone out as. ([#431](https://github.com/tox-dev/toml-fmt/pull/431)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 2.27.0 (2026-08-03)

<a name="release-pyproject-fmt-2-27-0"></a>

<a name="release-pyproject-fmt-2-27-0-added"></a>

#### Added

- **`project.version` is checked, and a value PEP 440 cannot read is named.**
  ([#428](https://github.com/tox-dev/toml-fmt/pull/428)) by [@gaborbernat](https://github.com/gaborbernat).

### 2.26.0 (2026-07-27)

<a name="release-pyproject-fmt-2-26-0"></a>

<a name="release-pyproject-fmt-2-26-0-added"></a>

#### Added

- **`[tool.pyproject-fmt]` orders its own keys.** ([#421](https://github.com/tox-dev/toml-fmt/pull/421)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 2.25.4 (2026-07-26)

<a name="release-pyproject-fmt-2-25-4"></a>

<a name="release-pyproject-fmt-2-25-4-packaging"></a>

#### Packaging

- **The source distribution carries only git-tracked sources.** A stray build artifact ended up in the sdist.
  ([#419](https://github.com/tox-dev/toml-fmt/pull/419)) by [@gaborbernat](https://github.com/gaborbernat).

### 2.25.3 (2026-07-13)

<a name="release-pyproject-fmt-2-25-3"></a>

<a name="release-pyproject-fmt-2-25-3-fixed"></a>

#### Fixed

- **A marker written after a URL keeps the space PEP 508 requires.** Without it the `;` reads as part of the URI.
  ([#410](https://github.com/tox-dev/toml-fmt/pull/410)) by [@gaborbernat](https://github.com/gaborbernat).

### 2.25.2 (2026-07-09)

<a name="release-pyproject-fmt-2-25-2"></a>

<a name="release-pyproject-fmt-2-25-2-fixed"></a>

#### Fixed

- **`sub_table_spacing` and `separate_root_table` honor a value of more than one blank line.**
  ([#403](https://github.com/tox-dev/toml-fmt/pull/403)) by [@gaborbernat](https://github.com/gaborbernat).

<a name="release-pyproject-fmt-2-25-2-documentation"></a>

#### Documentation

- **The formatter itself produces every formatting example**, so the docs cannot describe something the tool does not
  do. ([#397](https://github.com/tox-dev/toml-fmt/pull/397)) by [@gaborbernat](https://github.com/gaborbernat).

### 2.25.1 (2026-06-25)

<a name="release-pyproject-fmt-2-25-1"></a>

<a name="release-pyproject-fmt-2-25-1-fixed"></a>

#### Fixed

- **A key the file wrote as a comment reorders and comments back out without losing what it said.**
  ([#391](https://github.com/tox-dev/toml-fmt/pull/391)) by [@gaborbernat](https://github.com/gaborbernat).
- **An array entry keeps the comments and blank lines around it when an inline table is reordered.**
  ([#392](https://github.com/tox-dev/toml-fmt/pull/392)) by [@gaborbernat](https://github.com/gaborbernat).
- **A line-ending backslash inside a multi-line basic string reads as the continuation it is.**
  ([#393](https://github.com/tox-dev/toml-fmt/pull/393)) by [@gaborbernat](https://github.com/gaborbernat).
- **`base_python_file` sorts with the other tox environment keys.**
  ([#386](https://github.com/tox-dev/toml-fmt/pull/386)) by [@edgarrmondragon](https://github.com/edgarrmondragon).

### 2.25.0 (2026-06-17)

<a name="release-pyproject-fmt-2-25-0"></a>

<a name="release-pyproject-fmt-2-25-0-added"></a>

#### Added

- **A key the file wrote as a comment formats with the table it belongs to**, then comments back out. A
  `# dependencies = [...]` left in place for later keeps its position and its shape.
  ([#381](https://github.com/tox-dev/toml-fmt/pull/381), [#383](https://github.com/tox-dev/toml-fmt/pull/383)) by
  [@gaborbernat](https://github.com/gaborbernat).
- **`# Group:` markers scope reordering.** Related keys held under one marker sort among themselves, and nothing crosses
  a marker. ([#380](https://github.com/tox-dev/toml-fmt/pull/380)) by [@gaborbernat](https://github.com/gaborbernat).
- **`uv tool install pyproject-fmt` works.** ([#382](https://github.com/tox-dev/toml-fmt/pull/382)) by
  [@bn-andrew](https://github.com/bn-andrew).

<a name="release-pyproject-fmt-2-25-0-fixed"></a>

#### Fixed

- **A table the file wrote empty survives when its sub-table stays written out.**
  ([#385](https://github.com/tox-dev/toml-fmt/pull/385)) by [@gaborbernat](https://github.com/gaborbernat).

### 2.24.1 (2026-06-13)

<a name="release-pyproject-fmt-2-24-1"></a>

### 2.24.0 (2026-06-11)

<a name="release-pyproject-fmt-2-24-0"></a>

<a name="release-pyproject-fmt-2-24-0-changes"></a>

#### Changes

- **A setuptools `build-system.requires` drops a redundant `wheel`.** Setuptools has bundled `bdist_wheel` since 70.1.
  ([#373](https://github.com/tox-dev/toml-fmt/pull/373)) by [@gaborbernat](https://github.com/gaborbernat).

### 2.23.0 (2026-05-30)

<a name="release-pyproject-fmt-2-23-0"></a>

<a name="release-pyproject-fmt-2-23-0-documentation"></a>

#### Documentation

- **The formatting reference is rewritten**, and now gives the key order and array policy of every tool it knows.
  ([#366](https://github.com/tox-dev/toml-fmt/pull/366)) by [@gaborbernat](https://github.com/gaborbernat).

### 2.22.0 (2026-05-30)

<a name="release-pyproject-fmt-2-22-0"></a>

<a name="release-pyproject-fmt-2-22-0-added"></a>

#### Added

- **Thirty tool tables get a format of their own.** `autopep8`, `bandit`, `black`, `bumpversion`, `check-manifest`,
  `cibuildwheel`, `codespell`, `commitizen`, `deptry`, `djlint`, `docformatter`, `hatch`, `interrogate`, `isort`,
  `maturin`, `mypy`, `pdm`, `poetry`, `pylint`, `pyright` and `basedpyright`, `pyrefly`, `pytest`, `scikit-build`,
  `semantic_release`, `setuptools` and `setuptools_scm`, `towncrier`, `tox`, `ty`, `vulture`, `yapf`. Each gets the key
  order its own documentation gives, and each array is sorted or left in place depending on whether its order carries
  meaning. ([#324](https://github.com/tox-dev/toml-fmt/pull/324) through
  [#354](https://github.com/tox-dev/toml-fmt/pull/354)) by [@gaborbernat](https://github.com/gaborbernat).
- **`sub_table_spacing` and `separate_root_table` set the blank lines between tables.**
  ([#319](https://github.com/tox-dev/toml-fmt/pull/319)) by [@gaborbernat](https://github.com/gaborbernat).

<a name="release-pyproject-fmt-2-22-0-fixed"></a>

#### Fixed

- **A triple-quoted literal string survives formatting.** ([#356](https://github.com/tox-dev/toml-fmt/pull/356)) by
  [@gaborbernat](https://github.com/gaborbernat).
- **`_build_cli` is back for consumers that called it.** ([#361](https://github.com/tox-dev/toml-fmt/pull/361)) by
  [@gaborbernat](https://github.com/gaborbernat).

<a name="release-pyproject-fmt-2-22-0-packaging"></a>

#### Packaging

- **The wheels are self-contained**, so installing one no longer pulls a matching `toml-fmt-common` release.
  ([#363](https://github.com/tox-dev/toml-fmt/pull/363)) by [@gaborbernat](https://github.com/gaborbernat).
- **The source distribution carries its license file.** ([#313](https://github.com/tox-dev/toml-fmt/pull/313)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 2.21.2 (2026-05-05)

<a name="release-pyproject-fmt-2-21-2"></a>

<a name="release-pyproject-fmt-2-21-2-added"></a>

#### Added

- **Free-threaded Python wheels.** ([#307](https://github.com/tox-dev/toml-fmt/pull/307)) by
  [@gaborbernat](https://github.com/gaborbernat).

<a name="release-pyproject-fmt-2-21-2-fixed"></a>

#### Fixed

- **A table the file wrote empty no longer disappears from a lookup.**
  ([#304](https://github.com/tox-dev/toml-fmt/pull/304)) by [@gaborbernat](https://github.com/gaborbernat).

### 2.21.1 (2026-04-13)

<a name="release-pyproject-fmt-2-21-1"></a>

<a name="release-pyproject-fmt-2-21-1-fixed"></a>

#### Fixed

- **Sorting an array whose first value sits on the bracket line writes valid TOML.**
  ([#293](https://github.com/tox-dev/toml-fmt/pull/293)) by [@gaborbernat](https://github.com/gaborbernat).

### 2.21.0 (2026-03-31)

<a name="release-pyproject-fmt-2-21-0"></a>

<a name="release-pyproject-fmt-2-21-0-added"></a>

#### Added

- **`[tool.pixi]` gets a format of its own.** ([#283](https://github.com/tox-dev/toml-fmt/pull/283)) by
  [@gaborbernat](https://github.com/gaborbernat).

<a name="release-pyproject-fmt-2-21-0-fixed"></a>

#### Fixed

- **`--check` and `--stdout` no longer ask for a writable file.** Neither mode writes anything, so a read-only file
  failed on a permission neither needs. ([#278](https://github.com/tox-dev/toml-fmt/pull/278)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 2.20.0 (2026-03-18)

<a name="release-pyproject-fmt-2-20-0"></a>

<a name="release-pyproject-fmt-2-20-0-changes"></a>

#### Changes

- **`bandit` counts as a linter**, so its table sits with the other linters.
  ([#276](https://github.com/tox-dev/toml-fmt/pull/276)) by [@gaborbernat](https://github.com/gaborbernat).

### 2.19.0 (2026-03-16)

<a name="release-pyproject-fmt-2-19-0"></a>

<a name="release-pyproject-fmt-2-19-0-changes"></a>

#### Changes

- **Type checkers sort after linters**, so the file reads in the order a project runs them.
  ([#274](https://github.com/tox-dev/toml-fmt/pull/274)) by [@gaborbernat](https://github.com/gaborbernat).

### 2.18.1 (2026-03-03)

<a name="release-pyproject-fmt-2-18-1"></a>

<a name="release-pyproject-fmt-2-18-1-fixed"></a>

#### Fixed

- **A value that is not an array no longer panics an array rule.**
  ([#266](https://github.com/tox-dev/toml-fmt/pull/266)) by [@gaborbernat](https://github.com/gaborbernat).

### 2.18.0 (2026-03-03)

<a name="release-pyproject-fmt-2-18-0"></a>

<a name="release-pyproject-fmt-2-18-0-added"></a>

#### Added

- **Inline table keys follow the order the kind of table asks for.**
  ([#264](https://github.com/tox-dev/toml-fmt/pull/264)) by [@gaborbernat](https://github.com/gaborbernat).

### 2.17.0 (2026-03-01)

<a name="release-pyproject-fmt-2-17-0"></a>

<a name="release-pyproject-fmt-2-17-0-added"></a>

#### Added

- **One config file steers every formatter.** Settings written once in a shared file reach each tool built on
  `toml-fmt-common`, so a project stops repeating them per tool. ([#258](https://github.com/tox-dev/toml-fmt/pull/258))
  by [@gaborbernat](https://github.com/gaborbernat).

<a name="release-pyproject-fmt-2-17-0-fixed"></a>

#### Fixed

- **The parser keeps up with tombi v0.8.0.** ([#259](https://github.com/tox-dev/toml-fmt/pull/259)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 2.16.2 (2026-02-23)

<a name="release-pyproject-fmt-2-16-2"></a>

<a name="release-pyproject-fmt-2-16-2-fixed"></a>

#### Fixed

- **A classifier the trove list does not hold no longer panics the formatter.**
  ([#241](https://github.com/tox-dev/toml-fmt/pull/241)) by [@Nicolaus93](https://github.com/Nicolaus93) and
  [@gaborbernat](https://github.com/gaborbernat).
- **An environment whose name holds a dot is one environment.** ([#234](https://github.com/tox-dev/toml-fmt/pull/234))
  by [@gaborbernat](https://github.com/gaborbernat).

### 2.16.1 (2026-02-18)

<a name="release-pyproject-fmt-2-16-1"></a>

<a name="release-pyproject-fmt-2-16-1-changes"></a>

#### Changes

- **`authors` and `maintainers` keep the order the file wrote them in.** That order is published metadata and says who
  led the work, so sorting it changed what the package claimed. ([#228](https://github.com/tox-dev/toml-fmt/issues/228),
  [#229](https://github.com/tox-dev/toml-fmt/pull/229)) by [@gaborbernat](https://github.com/gaborbernat).

<a name="release-pyproject-fmt-2-16-1-documentation"></a>

#### Documentation

- **`column_width` documents how it wraps a string.** ([#223](https://github.com/tox-dev/toml-fmt/pull/223)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 2.16.0 (2026-02-12)

<a name="release-pyproject-fmt-2-16-0"></a>

<a name="release-pyproject-fmt-2-16-0-fixed"></a>

#### Fixed

- **A key the file quoted reads as the one name it spells.** A literal-quoted key and a basic-quoted one name the same
  key, and sorting told them apart. ([#217](https://github.com/tox-dev/toml-fmt/issues/217),
  [#219](https://github.com/tox-dev/toml-fmt/pull/219)) by [@gaborbernat](https://github.com/gaborbernat).

### 2.15.3 (2026-02-11)

<a name="release-pyproject-fmt-2-15-3"></a>

<a name="release-pyproject-fmt-2-15-3-added"></a>

#### Added

- **`skip_wrap_for_keys` holds a value on one line.** A key whose value carries meaning that line breaks would obscure,
  such as a command line, can now be named. ([#216](https://github.com/tox-dev/toml-fmt/pull/216)) by
  [@gaborbernat](https://github.com/gaborbernat).

<a name="release-pyproject-fmt-2-15-3-fixed"></a>

#### Fixed

- **Quote styles agree when keys sort.** ([#215](https://github.com/tox-dev/toml-fmt/pull/215)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 2.15.2 (2026-02-10)

<a name="release-pyproject-fmt-2-15-2"></a>

<a name="release-pyproject-fmt-2-15-2-fixed"></a>

#### Fixed

- **A comment closing an array's last line survives a sort.** ([#208](https://github.com/tox-dev/toml-fmt/pull/208)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 2.15.1 (2026-02-10)

<a name="release-pyproject-fmt-2-15-1"></a>

<a name="release-pyproject-fmt-2-15-1-fixed"></a>

#### Fixed

- **An array written on one line with a trailing comment stays on one line.**
  ([#204](https://github.com/tox-dev/toml-fmt/pull/204)) by [@gaborbernat](https://github.com/gaborbernat).
- **A table the file wrote empty comes back as `{}` rather than disappearing.**
  ([#203](https://github.com/tox-dev/toml-fmt/pull/203)) by [@gaborbernat](https://github.com/gaborbernat).

### 2.15.0 (2026-02-09)

<a name="release-pyproject-fmt-2-15-0"></a>

<a name="release-pyproject-fmt-2-15-0-added"></a>

#### Added

- **`[tool.coverage]` gets a key order, and its list-valued keys sort.**
  ([#199](https://github.com/tox-dev/toml-fmt/pull/199)) by [@gaborbernat](https://github.com/gaborbernat).

<a name="release-pyproject-fmt-2-15-0-fixed"></a>

#### Fixed

- **Collapsing an array of tables carries the comments written inside it.**
  ([#198](https://github.com/tox-dev/toml-fmt/pull/198)) by [@gaborbernat](https://github.com/gaborbernat).

### 2.14.2 (2026-02-09)

<a name="release-pyproject-fmt-2-14-2"></a>

<a name="release-pyproject-fmt-2-14-2-fixed"></a>

#### Fixed

- **Raising test coverage to 98% turned up several defects.** ([#194](https://github.com/tox-dev/toml-fmt/pull/194)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 2.14.1 (2026-02-09)

<a name="release-pyproject-fmt-2-14-1"></a>

<a name="release-pyproject-fmt-2-14-1-added"></a>

#### Added

- **`[tool.uv]` gets a format of its own.** ([#169](https://github.com/tox-dev/toml-fmt/pull/169)) by
  [@gaborbernat](https://github.com/gaborbernat).

<a name="release-pyproject-fmt-2-14-1-fixed"></a>

#### Fixed

- **A table header survives formatting.** ([#192](https://github.com/tox-dev/toml-fmt/pull/192)) by
  [@gaborbernat](https://github.com/gaborbernat).
- **A comment written beside a string in an array stays with it.**
  ([#191](https://github.com/tox-dev/toml-fmt/pull/191)) by [@gaborbernat](https://github.com/gaborbernat).
- **A comment holding a double quote inside an array reads as one comment.**
  ([#189](https://github.com/tox-dev/toml-fmt/pull/189)) by [@gaborbernat](https://github.com/gaborbernat).
- **An entry with a leading comment sorts by its value.** It sorted by the comment above it, so a comment decided where
  the value went. ([#188](https://github.com/tox-dev/toml-fmt/pull/188)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 2.14.0 (2026-02-08)

<a name="release-pyproject-fmt-2-14-0"></a>

### 2.13.0 (2026-02-07)

<a name="release-pyproject-fmt-2-13-0"></a>

<a name="release-pyproject-fmt-2-13-0-changes"></a>

#### Changes

- **Strings prefer double quotes, and single quotes where that saves an escape.**
  ([#162](https://github.com/tox-dev/toml-fmt/pull/162)) by [@gaborbernat](https://github.com/gaborbernat).
- **An array without a trailing comma stays on one line when it fits; a trailing comma expands it.**
  ([#164](https://github.com/tox-dev/toml-fmt/pull/164)) by [@gaborbernat](https://github.com/gaborbernat).
- **Unhandled table keys sort alphabetically.** ([#164](https://github.com/tox-dev/toml-fmt/pull/164)) by
  [@gaborbernat](https://github.com/gaborbernat).

<a name="release-pyproject-fmt-2-13-0-fixed"></a>

#### Fixed

- **`expand_tables` reaches a table written any number of levels down.**
  ([#146](https://github.com/tox-dev/toml-fmt/issues/146), [#160](https://github.com/tox-dev/toml-fmt/pull/160)) by
  [@gaborbernat](https://github.com/gaborbernat).
- **A comment written above a table header stays with that table.**
  ([#158](https://github.com/tox-dev/toml-fmt/pull/158)) by [@gaborbernat](https://github.com/gaborbernat).
- **An SPDX license expression reads by the operators the spec defines.**
  ([#156](https://github.com/tox-dev/toml-fmt/pull/156)) by [@alexfikl](https://github.com/alexfikl).
- **Sorting an array keeps comments with their values.** ([#164](https://github.com/tox-dev/toml-fmt/pull/164)) by
  [@gaborbernat](https://github.com/gaborbernat).

<a name="release-pyproject-fmt-2-13-0-documentation"></a>

#### Documentation

- **The formatting reference is rewritten, and it corrects the array behavior the old text described.**
  ([#164](https://github.com/tox-dev/toml-fmt/pull/164)) by [@gaborbernat](https://github.com/gaborbernat).

### 2.12.1 (2026-01-31)

<a name="release-pyproject-fmt-2-12-1"></a>

<a name="release-pyproject-fmt-2-12-1-fixed"></a>

#### Fixed

- **`expand_tables` reaches a named sub-table.** A setting naming one matched nothing.
  ([#148](https://github.com/tox-dev/toml-fmt/pull/148)) by [@gaborbernat](https://github.com/gaborbernat).

### 2.12.0 (2026-01-31)

<a name="release-pyproject-fmt-2-12-0"></a>

<a name="release-pyproject-fmt-2-12-0-added"></a>

#### Added

- **`table_format`, `expand_tables` and `collapse_tables` choose how tables are written.** A table can fold into its
  parent as dotted keys or stay under a header of its own, per project or per table.
  ([#141](https://github.com/tox-dev/toml-fmt/pull/141), [#142](https://github.com/tox-dev/toml-fmt/pull/142)) by
  [@gaborbernat](https://github.com/gaborbernat).
- **An extra name in `optional-dependencies` reads the way PEP 685 spells one.**
  ([#135](https://github.com/tox-dev/toml-fmt/pull/135)) by [@gaborbernat](https://github.com/gaborbernat).
- **An SPDX operator in a license expression takes the case the spec gives it.**
  ([#136](https://github.com/tox-dev/toml-fmt/pull/136)) by [@gaborbernat](https://github.com/gaborbernat).
- **Keywords and classifiers say each entry once.** ([#133](https://github.com/tox-dev/toml-fmt/pull/133)) by
  [@gaborbernat](https://github.com/gaborbernat).
- **A PEP 794 `import-names` entry reads the way the spec spells one.**
  ([#131](https://github.com/tox-dev/toml-fmt/pull/131)) by [@gaborbernat](https://github.com/gaborbernat).
- **`[tool.nuitka]` and `[tool.typos]` get a format of their own.**
  ([#123](https://github.com/tox-dev/toml-fmt/pull/123)) by [@kdeldycke](https://github.com/kdeldycke).

<a name="release-pyproject-fmt-2-12-0-changes"></a>

#### Changes

- **Sub-tables of one tool sort among themselves**, so an unrelated tool cannot land between them.
  ([#139](https://github.com/tox-dev/toml-fmt/pull/139)) by [@gaborbernat](https://github.com/gaborbernat).
- **`[[project.authors]]` folds into an inline array.** ([#137](https://github.com/tox-dev/toml-fmt/pull/137)) by
  [@gaborbernat](https://github.com/gaborbernat).
- **`authors` and `maintainers` sort.** ([#134](https://github.com/tox-dev/toml-fmt/pull/134)) by
  [@gaborbernat](https://github.com/gaborbernat).

<a name="release-pyproject-fmt-2-12-0-fixed"></a>

#### Fixed

- **A comment written before a table header stays with the table it was written for.**
  ([#140](https://github.com/tox-dev/toml-fmt/pull/140)) by [@gaborbernat](https://github.com/gaborbernat).
- **A multi-line string using a line continuation no longer crashes the formatter.**
  ([#132](https://github.com/tox-dev/toml-fmt/pull/132)) by [@gaborbernat](https://github.com/gaborbernat).
- **A literal string holding what looks like an invalid escape survives.** `'C:\path'` came back rewritten.
  ([#130](https://github.com/tox-dev/toml-fmt/pull/130)) by [@gaborbernat](https://github.com/gaborbernat).
- **`requires-python` with the `~=` operator reads.** ([#128](https://github.com/tox-dev/toml-fmt/pull/128)) by
  [@gaborbernat](https://github.com/gaborbernat).
- **A build requirement survives where two entries name the same package.** One of them was dropped.
  ([#127](https://github.com/tox-dev/toml-fmt/pull/127)) by [@gaborbernat](https://github.com/gaborbernat).

<a name="release-pyproject-fmt-2-12-0-documentation"></a>

#### Documentation

- **The docs spell out the formatting principles and every normalization**, and `README.rst` comes from them so the two
  cannot drift. ([#144](https://github.com/tox-dev/toml-fmt/pull/144),
  [#145](https://github.com/tox-dev/toml-fmt/pull/145)) by [@gaborbernat](https://github.com/gaborbernat).

### 2.11.1 (2026-01-07)

<a name="release-pyproject-fmt-2-11-1"></a>

<a name="release-pyproject-fmt-2-11-1-added"></a>

#### Added

- **PEP 794 `import-names` support.** ([#109](https://github.com/tox-dev/toml-fmt/pull/109)) by
  [@henryiii](https://github.com/henryiii).

<a name="release-pyproject-fmt-2-11-1-fixed"></a>

#### Fixed

- **A version written in parentheses reads.** `(>=1.0)` was rejected, so a requirement written that way came back
  unformatted. ([#122](https://github.com/tox-dev/toml-fmt/pull/122)) by [@jamesbursa](https://github.com/jamesbursa).

<a name="release-pyproject-fmt-2-11-1-documentation"></a>

#### Documentation

- **The tool names and the links between pages are correct.** ([#105](https://github.com/tox-dev/toml-fmt/pull/105),
  [#106](https://github.com/tox-dev/toml-fmt/pull/106)) by [@gaborbernat](https://github.com/gaborbernat) and
  [@jorisboeye](https://github.com/jorisboeye).

### 2.11.0 (2025-10-15)

<a name="release-pyproject-fmt-2-11-0"></a>

<a name="release-pyproject-fmt-2-11-0-added"></a>

#### Added

- **`--no-generate-python-version-classifiers` turns the generated classifiers off.** A project that lists them by hand
  can keep its own list. ([#102](https://github.com/tox-dev/toml-fmt/pull/102)) by
  [@gaborbernat](https://github.com/gaborbernat).

<a name="release-pyproject-fmt-2-11-0-fixed"></a>

#### Fixed

- **A version suffix reads only as far as the specification allows.**
  ([#101](https://github.com/tox-dev/toml-fmt/pull/101)) by [@gaborbernat](https://github.com/gaborbernat).

### 2.10.0 (2025-10-09)

<a name="release-pyproject-fmt-2-10-0"></a>

<a name="release-pyproject-fmt-2-10-0-added"></a>

#### Added

- **More type checkers count as top-level tables**, so their sections sort with the rest rather than falling to the end.
  ([#98](https://github.com/tox-dev/toml-fmt/pull/98)) by [@browniebroke](https://github.com/browniebroke).

### 2.9.0 (2025-10-08)

<a name="release-pyproject-fmt-2-9-0"></a>

<a name="release-pyproject-fmt-2-9-0-added"></a>

#### Added

- **Python 3.14 wheels.** ([#96](https://github.com/tox-dev/toml-fmt/pull/96)) by
  [@gaborbernat](https://github.com/gaborbernat).
- **End support for Python 3.9.** ([#96](https://github.com/tox-dev/toml-fmt/pull/96)) by
  [@gaborbernat](https://github.com/gaborbernat).

<a name="release-pyproject-fmt-2-9-0-changes"></a>

#### Changes

- **`[tool.uv]` sorts higher in the file**, with the tools that install rather than the ones that lint.
  ([#97](https://github.com/tox-dev/toml-fmt/pull/97)) by [@browniebroke](https://github.com/browniebroke).

### 2.8.0 (2025-10-08)

<a name="release-pyproject-fmt-2-8-0"></a>

<a name="release-pyproject-fmt-2-8-0-changes"></a>

#### Changes

- **The default supported Python range is 3.10 to 3.14.** The generated classifiers follow it where `requires-python`
  says nothing. ([#94](https://github.com/tox-dev/toml-fmt/pull/94)) by [@gaborbernat](https://github.com/gaborbernat).

<a name="release-pyproject-fmt-2-8-0-fixed"></a>

#### Fixed

- **A pre-release label reads the way PEP 440 spells one.** ([#89](https://github.com/tox-dev/toml-fmt/pull/89)) by
  [@adamchainz](https://github.com/adamchainz).

### 2.7.0 (2025-10-01)

<a name="release-pyproject-fmt-2-7-0"></a>

<a name="release-pyproject-fmt-2-7-0-changes"></a>

#### Changes

- **PEP 508 parsing moved in-tree.** The upstream crate was deprecated, so requirement parsing no longer depends on it.
  ([#84](https://github.com/tox-dev/toml-fmt/pull/84)) by [@gaborbernat](https://github.com/gaborbernat).

<a name="release-pyproject-fmt-2-7-0-fixed"></a>

#### Fixed

- **A sorted table keeps the empty lines the file wrote around it.**
  ([#63](https://github.com/tox-dev/toml-fmt/pull/63)) by [@ddeepwell](https://github.com/ddeepwell).
- **The space before a file extension survives.** ([#76](https://github.com/tox-dev/toml-fmt/pull/76)) by
  [@hugovk](https://github.com/hugovk).

### 2.6.0 (2025-05-19)

<a name="release-pyproject-fmt-2-6-0"></a>

<a name="release-pyproject-fmt-2-6-0-packaging"></a>

#### Packaging

- **One `abi3-py39` wheel per platform**, rather than one per interpreter version.
  ([#58](https://github.com/tox-dev/toml-fmt/pull/58)) by [@gaborbernat](https://github.com/gaborbernat).

### 2.5.1 (2025-02-18)

<a name="release-pyproject-fmt-2-5-1"></a>

<a name="release-pyproject-fmt-2-5-1-fixed"></a>

#### Fixed

- **Every part of a dotted key counts.** A key such as `tool.ruff.lint` was read as far as the first segment, so the
  rest decided nothing. ([#31](https://github.com/tox-dev/toml-fmt/pull/31)) by
  [@adamchainz](https://github.com/adamchainz).

### 2.5.0 (2024-10-30)

<a name="release-pyproject-fmt-2-5-0"></a>

<a name="release-pyproject-fmt-2-5-0-added"></a>

#### Added

- **PEP 735 `dependency-groups` support.** ([#16](https://github.com/tox-dev/toml-fmt/pull/16)) by
  [@browniebroke](https://github.com/browniebroke) and [@gaborbernat](https://github.com/gaborbernat).

### 2.4.3 (2024-10-17)

<a name="release-pyproject-fmt-2-4-3"></a>

<a name="release-pyproject-fmt-2-4-3-packaging"></a>

#### Packaging

- **Python 3.10 installs `tomli`.** The wheel needed it to read a config file and did not declare it.
  ([#9](https://github.com/tox-dev/toml-fmt/pull/9)) by [@gaborbernat](https://github.com/gaborbernat).

### 2.4.2 (2024-10-17)

<a name="release-pyproject-fmt-2-4-2"></a>

### 2.4.1 (2024-10-17)

<a name="release-pyproject-fmt-2-4-1"></a>

### 2.4.0 (2024-10-17)

<a name="release-pyproject-fmt-2-4-0"></a>

<a name="release-pyproject-fmt-2-4-0-changes"></a>

#### Changes

- **The project moved into the `toml-fmt` workspace**, alongside `tox-toml-fmt` and the Python code they share, so a
  change to that shared code reaches both tools in one release. ([#1](https://github.com/tox-dev/toml-fmt/pull/1)) by
  [@gaborbernat](https://github.com/gaborbernat).

## tox-toml-fmt

### 1.11.1 (2026-10-10)

<a name="release-tox-toml-fmt-1-11-1"></a>

<a name="release-tox-toml-fmt-1-11-1-added"></a>

#### Added

- **Require 3.11 and default to 3.15.** ([#480](https://github.com/tox-dev/toml-fmt/pull/480)) by
  [@gaborbernat](https://github.com/gaborbernat).

<a name="release-tox-toml-fmt-1-11-1-fixed"></a>

#### Fixed

- **Format every input after an earlier one changes.** ([#479](https://github.com/tox-dev/toml-fmt/pull/479)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 1.11.0 (2026-10-02)

<a name="release-tox-toml-fmt-1-11-0"></a>

### 1.10.3 (2026-09-01)

<a name="release-tox-toml-fmt-1-10-3"></a>

<a name="release-tox-toml-fmt-1-10-3-fixed"></a>

#### Fixed

- **The test suite the source distribution ships passes from the unpacked sdist.** The backend read the project metadata
  from beside itself, and the sdist keeps a copy of that file next to the tests, a directory below the `pyproject.toml`
  it belongs to, so eight tests ended on the file it could not find. It reads the metadata from either place.
  ([#454](https://github.com/tox-dev/toml-fmt/issues/454), [#457](https://github.com/tox-dev/toml-fmt/pull/457)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 1.10.2 (2026-08-31)

<a name="release-tox-toml-fmt-1-10-2"></a>

<a name="release-tox-toml-fmt-1-10-2-fixed"></a>

#### Fixed

- **A build from the source distribution runs on what that distribution carries.** 1.10.0 named toml-fmt-common as a
  dependency and resolved it from PyPI, where the newest release dates from May; 1.10.1 dropped the dependency before
  the sdist carried the sources, so a build from it failed on the import. This release carries them, and its metadata
  requires nothing. ([#450](https://github.com/tox-dev/toml-fmt/issues/450),
  [#452](https://github.com/tox-dev/toml-fmt/pull/452)) by [@gaborbernat](https://github.com/gaborbernat).

  The wheels held their vendored copy throughout, so an install that takes a wheel was never affected.

### 1.10.1 (2026-08-31)

<a name="release-tox-toml-fmt-1-10-1"></a>

<a name="release-tox-toml-fmt-1-10-1-fixed"></a>

#### Fixed

- **The source distribution no longer names toml-fmt-common as a dependency.** The newest release of that package on
  PyPI dates from May, months behind this one, so a build from the sdist ran an old settings reader against the current
  formatter and failed a large part of its own test suite. ([#450](https://github.com/tox-dev/toml-fmt/issues/450),
  [#451](https://github.com/tox-dev/toml-fmt/pull/451)) by [@gaborbernat](https://github.com/gaborbernat).

<a name="release-tox-toml-fmt-1-10-1-packaging"></a>

#### Packaging

- **This release's source distribution carries no toml-fmt-common, so a build from it fails on the import.**
  [1.10.2](https://github.com/tox-dev/toml-fmt/releases/tag/tox-toml-fmt/1.10.2) carries it. The wheels hold their
  vendored copy and install as before. ([#452](https://github.com/tox-dev/toml-fmt/pull/452)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 1.10.0 (2026-08-31)

<a name="release-tox-toml-fmt-1-10-0"></a>

<a name="release-tox-toml-fmt-1-10-0-changes"></a>

#### Changes

- **`env_list` keeps the order the file wrote it in.** That list is the order tox runs the environments in: it feeds the
  sequential runner and decides where a fail-fast run stops, so sorting it changed what the file said. `--pin-env` still
  moves the environments it names to the front, of both the list and the tables.
  ([#448](https://github.com/tox-dev/toml-fmt/pull/448)) by [@gaborbernat](https://github.com/gaborbernat).
- **A string measures from the start of its key.** A long key can be what pushes a value past `column_width`; measuring
  the value alone left lines running past the column the setting asks for.
  ([#448](https://github.com/tox-dev/toml-fmt/pull/448)) by [@gaborbernat](https://github.com/gaborbernat).
- **Deep input comes back as an error rather than a crash.** Reading a value, writing it and dropping it each walk it by
  calling themselves, so the model caps nesting at 256 levels. A 12,000-deep value used to end the process.
  ([#448](https://github.com/tox-dev/toml-fmt/pull/448)) by [@gaborbernat](https://github.com/gaborbernat).

<a name="release-tox-toml-fmt-1-10-0-fixed"></a>

#### Fixed

- **An environment whose name the file quoted follows `env_list`.** `[env."3.14"]` is the environment the list names as
  `"3.14"`, and comparing the quoted key against the plain name never said so, so its table fell in with the
  environments the list does not name at all. ([#448](https://github.com/tox-dev/toml-fmt/pull/448)) by
  [@gaborbernat](https://github.com/gaborbernat).
- **A `deps` list holding a pip option, path or URL keeps its order.** pip reads that list the way it reads a
  requirements file, where a later `--index-url` replaces the one before it, so the order carries meaning. Every
  requirement beside such an entry still normalizes. ([#448](https://github.com/tox-dev/toml-fmt/pull/448)) by
  [@gaborbernat](https://github.com/gaborbernat).
- **A `set_env` table keeps the order the file wrote it in.** tox reads the table in order, so a key written after
  `file` overrides what that file said and a key written before it does not; alphabetizing the table changed which value
  won. ([#448](https://github.com/tox-dev/toml-fmt/pull/448)) by [@gaborbernat](https://github.com/gaborbernat).
- **`use_develop = true` beside a `package` key gives `package` the mode the environment ran with.** tox reads
  `use_develop` first and installs an editable package whatever `package` says; dropping the older key and keeping
  `package = "sdist"` changed the install. ([#448](https://github.com/tox-dev/toml-fmt/pull/448)) by
  [@gaborbernat](https://github.com/gaborbernat).
- **A nested array closes its last member with the comma the outer form writes.**
  ([#448](https://github.com/tox-dev/toml-fmt/pull/448)) by [@gaborbernat](https://github.com/gaborbernat).
- **A literal string sorts with the values around it.** `'zz'` and `'aa'` held their order while the same values in
  double quotes sorted. ([#448](https://github.com/tox-dev/toml-fmt/pull/448)) by
  [@gaborbernat](https://github.com/gaborbernat).
- **A comment written before a member's comma stays on that member's line.** The comma is what says which member a
  comment belongs to: one written before it closes that member's line, one written after it leads the next member. Both
  used to end up on a line of their own, leaving the comma stranded below.
  ([#448](https://github.com/tox-dev/toml-fmt/pull/448)) by [@gaborbernat](https://github.com/gaborbernat).

<a name="release-tox-toml-fmt-1-10-0-performance"></a>

#### Performance

- **The quadratic walks the old parse tree forced are gone.** 800 environments, each with one sub-table, took 18.6
  seconds and now take 0.12; scaling is close to linear. ([#448](https://github.com/tox-dev/toml-fmt/pull/448)) by
  [@gaborbernat](https://github.com/gaborbernat).

<a name="release-tox-toml-fmt-1-10-0-packaging"></a>

#### Packaging

- **This release's source distribution names a dependency on toml-fmt-common without carrying it, and the newest release
  of that package on PyPI dates from May.** A build from the sdist therefore runs an older settings reader against this
  formatter. [1.10.2](https://github.com/tox-dev/toml-fmt/releases/tag/tox-toml-fmt/1.10.2) carries the sources. The
  wheels hold their vendored copy and install as before. ([#450](https://github.com/tox-dev/toml-fmt/issues/450))

### 1.9.3 (2026-05-05)

<a name="release-tox-toml-fmt-1-9-3"></a>

<a name="release-tox-toml-fmt-1-9-3-added"></a>

#### Added

- **Free-threaded Python wheels.** ([#307](https://github.com/tox-dev/toml-fmt/pull/307)) by
  [@gaborbernat](https://github.com/gaborbernat).

<a name="release-tox-toml-fmt-1-9-3-fixed"></a>

#### Fixed

- **A table the file wrote empty no longer disappears from a lookup.**
  ([#304](https://github.com/tox-dev/toml-fmt/pull/304)) by [@gaborbernat](https://github.com/gaborbernat).

### 1.9.2 (2026-04-13)

<a name="release-tox-toml-fmt-1-9-2"></a>

<a name="release-tox-toml-fmt-1-9-2-fixed"></a>

#### Fixed

- **Sorting an array whose first value sits on the bracket line writes valid TOML.** The result did not parse.
  ([#293](https://github.com/tox-dev/toml-fmt/pull/293)) by [@gaborbernat](https://github.com/gaborbernat).

### 1.9.1 (2026-03-11)

<a name="release-tox-toml-fmt-1-9-1"></a>

<a name="release-tox-toml-fmt-1-9-1-fixed"></a>

#### Fixed

- **A `deps` entry naming a path or a tox substitution stays as written.** Normalizing `{tox_root}/pkg` or `./local` as
  a requirement rewrote something pip reads differently. ([#268](https://github.com/tox-dev/toml-fmt/pull/268)) by
  [@gaborbernat](https://github.com/gaborbernat).
- **A value that is not an array no longer panics an array rule.**
  ([#266](https://github.com/tox-dev/toml-fmt/pull/266)) by [@gaborbernat](https://github.com/gaborbernat).

### 1.9.0 (2026-03-03)

<a name="release-tox-toml-fmt-1-9-0"></a>

<a name="release-tox-toml-fmt-1-9-0-added"></a>

#### Added

- **Inline table keys follow the order the kind of table asks for.** A `{ replace = "ref", of = [...] }` reads in the
  order tox documents rather than the order it happened to be typed.
  ([#264](https://github.com/tox-dev/toml-fmt/pull/264)) by [@gaborbernat](https://github.com/gaborbernat).

### 1.8.0 (2026-03-01)

<a name="release-tox-toml-fmt-1-8-0"></a>

<a name="release-tox-toml-fmt-1-8-0-added"></a>

#### Added

- **One config file steers every formatter.** Settings written once in a shared file reach each tool built on
  `toml-fmt-common`, so a project stops repeating them per tool. ([#258](https://github.com/tox-dev/toml-fmt/pull/258))
  by [@gaborbernat](https://github.com/gaborbernat).

<a name="release-tox-toml-fmt-1-8-0-fixed"></a>

#### Fixed

- **The parser keeps up with tombi v0.8.0.** ([#259](https://github.com/tox-dev/toml-fmt/pull/259)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 1.7.2 (2026-02-18)

<a name="release-tox-toml-fmt-1-7-2"></a>

<a name="release-tox-toml-fmt-1-7-2-fixed"></a>

#### Fixed

- **An environment whose name holds a dot is one environment.** `[env."3.12-django"]` was read as a table two levels
  down, so its keys were ordered by the wrong rules. ([#234](https://github.com/tox-dev/toml-fmt/pull/234)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 1.7.1 (2026-02-18)

<a name="release-tox-toml-fmt-1-7-1"></a>

<a name="release-tox-toml-fmt-1-7-1-fixed"></a>

#### Fixed

- **Each environment keeps a section of its own.** Folding them together lost the grouping the file is built on.
  ([#231](https://github.com/tox-dev/toml-fmt/pull/231)) by [@gaborbernat](https://github.com/gaborbernat).

### 1.7.0 (2026-02-18)

<a name="release-tox-toml-fmt-1-7-0"></a>

<a name="release-tox-toml-fmt-1-7-0-added"></a>

#### Added

- **The key order matches `tox-ini-fmt`.** A project moving from `tox.ini` to `tox.toml` reads the same shape in both.
  ([#230](https://github.com/tox-dev/toml-fmt/pull/230)) by [@gaborbernat](https://github.com/gaborbernat).

<a name="release-tox-toml-fmt-1-7-0-documentation"></a>

#### Documentation

- **`column_width` documents how it wraps a string.** ([#223](https://github.com/tox-dev/toml-fmt/pull/223)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 1.6.0 (2026-02-12)

<a name="release-tox-toml-fmt-1-6-0"></a>

<a name="release-tox-toml-fmt-1-6-0-added"></a>

#### Added

- **`skip_wrap_for_keys` holds a value on one line.** A key whose value carries meaning that line breaks would obscure,
  such as a command line, can now be named. ([#216](https://github.com/tox-dev/toml-fmt/pull/216)) by
  [@gaborbernat](https://github.com/gaborbernat).

<a name="release-tox-toml-fmt-1-6-0-fixed"></a>

#### Fixed

- **A key the file quoted reads as the one name it spells.** A literal-quoted key and a basic-quoted one name the same
  key, and sorting told them apart, so the two forms landed in different places.
  ([#215](https://github.com/tox-dev/toml-fmt/pull/215), [#217](https://github.com/tox-dev/toml-fmt/issues/217),
  [#219](https://github.com/tox-dev/toml-fmt/pull/219)) by [@gaborbernat](https://github.com/gaborbernat).

### 1.5.4 (2026-02-11)

<a name="release-tox-toml-fmt-1-5-4"></a>

<a name="release-tox-toml-fmt-1-5-4-fixed"></a>

#### Fixed

- **A comment closing an array's last line survives a sort.** ([#208](https://github.com/tox-dev/toml-fmt/pull/208)) by
  [@gaborbernat](https://github.com/gaborbernat).
- **An array written on one line with a trailing comment stays on one line.**
  ([#204](https://github.com/tox-dev/toml-fmt/pull/204)) by [@gaborbernat](https://github.com/gaborbernat).
- **A table the file wrote empty comes back as `{}` rather than disappearing.**
  ([#203](https://github.com/tox-dev/toml-fmt/pull/203)) by [@gaborbernat](https://github.com/gaborbernat).

### 1.5.3 (2026-02-09)

<a name="release-tox-toml-fmt-1-5-3"></a>

<a name="release-tox-toml-fmt-1-5-3-fixed"></a>

#### Fixed

- **Collapsing an array of tables carries the comments written inside it.** They were dropped with the headers.
  ([#198](https://github.com/tox-dev/toml-fmt/pull/198)) by [@gaborbernat](https://github.com/gaborbernat).

### 1.5.2 (2026-02-09)

<a name="release-tox-toml-fmt-1-5-2"></a>

### 1.5.1 (2026-02-08)

<a name="release-tox-toml-fmt-1-5-1"></a>

<a name="release-tox-toml-fmt-1-5-1-fixed"></a>

#### Fixed

- **A table header survives formatting.** ([#192](https://github.com/tox-dev/toml-fmt/pull/192)) by
  [@gaborbernat](https://github.com/gaborbernat).
- **A comment written beside a string in an array stays with it.**
  ([#191](https://github.com/tox-dev/toml-fmt/pull/191)) by [@gaborbernat](https://github.com/gaborbernat).
- **A comment holding a double quote inside an array reads as one comment.** The quote was taken as the start of a
  string, so the rest of the line was misread. ([#189](https://github.com/tox-dev/toml-fmt/pull/189)) by
  [@gaborbernat](https://github.com/gaborbernat).
- **An entry with a leading comment sorts by its value.** It sorted by the comment above it, so a comment decided where
  the value went. ([#188](https://github.com/tox-dev/toml-fmt/pull/188)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 1.5.0 (2026-02-08)

<a name="release-tox-toml-fmt-1-5-0"></a>

### 1.4.0 (2026-02-07)

<a name="release-tox-toml-fmt-1-4-0"></a>

<a name="release-tox-toml-fmt-1-4-0-changes"></a>

#### Changes

- **Strings prefer double quotes, and single quotes where that saves an escape.**
  ([#162](https://github.com/tox-dev/toml-fmt/pull/162)) by [@gaborbernat](https://github.com/gaborbernat).
- **An array without a trailing comma stays on one line when it fits; a trailing comma expands it.**
  ([#164](https://github.com/tox-dev/toml-fmt/pull/164)) by [@gaborbernat](https://github.com/gaborbernat).
- **Unhandled table keys sort alphabetically.** ([#164](https://github.com/tox-dev/toml-fmt/pull/164)) by
  [@gaborbernat](https://github.com/gaborbernat).

<a name="release-tox-toml-fmt-1-4-0-fixed"></a>

#### Fixed

- **`expand_tables` reaches a table written any number of levels down.** A setting naming a deeply nested table matched
  nothing. ([#146](https://github.com/tox-dev/toml-fmt/issues/146),
  [#148](https://github.com/tox-dev/toml-fmt/pull/148), [#160](https://github.com/tox-dev/toml-fmt/pull/160)) by
  [@gaborbernat](https://github.com/gaborbernat).
- **A comment written above a table header stays with that table when the table has its own comments.**
  ([#158](https://github.com/tox-dev/toml-fmt/pull/158)) by [@gaborbernat](https://github.com/gaborbernat).
- **Sorting an array keeps comments with their values.** ([#164](https://github.com/tox-dev/toml-fmt/pull/164)) by
  [@gaborbernat](https://github.com/gaborbernat).
- **The command reports `tox.toml`, not `pyproject.toml`, as its input filename.**
  ([#164](https://github.com/tox-dev/toml-fmt/pull/164)) by [@gaborbernat](https://github.com/gaborbernat).

<a name="release-tox-toml-fmt-1-4-0-documentation"></a>

#### Documentation

- **The formatting reference is split into overview, configuration, and rules pages.**
  ([#164](https://github.com/tox-dev/toml-fmt/pull/164)) by [@gaborbernat](https://github.com/gaborbernat).

### 1.3.0 (2026-01-30)

<a name="release-tox-toml-fmt-1-3-0"></a>

<a name="release-tox-toml-fmt-1-3-0-added"></a>

#### Added

- **Environment tables follow the order in `env_list`.** ([#141](https://github.com/tox-dev/toml-fmt/pull/141)) by
  [@gaborbernat](https://github.com/gaborbernat).

<a name="release-tox-toml-fmt-1-3-0-changes"></a>

#### Changes

- **Sub-tables of one tool sort among themselves**, so an unrelated tool cannot land between them.
  ([#139](https://github.com/tox-dev/toml-fmt/pull/139)) by [@gaborbernat](https://github.com/gaborbernat).

<a name="release-tox-toml-fmt-1-3-0-fixed"></a>

#### Fixed

- **A comment written before a table header stays with that table.**
  ([#140](https://github.com/tox-dev/toml-fmt/pull/140)) by [@gaborbernat](https://github.com/gaborbernat).
- **A multi-line string using a line continuation no longer crashes the formatter.**
  ([#132](https://github.com/tox-dev/toml-fmt/pull/132)) by [@gaborbernat](https://github.com/gaborbernat).
- **A literal string holding what looks like an invalid escape survives.** `'C:\path'` came back rewritten.
  ([#130](https://github.com/tox-dev/toml-fmt/pull/130)) by [@gaborbernat](https://github.com/gaborbernat).
- **A build requirement survives where two entries name the same package.** One of them was dropped.
  ([#127](https://github.com/tox-dev/toml-fmt/pull/127)) by [@gaborbernat](https://github.com/gaborbernat).

<a name="release-tox-toml-fmt-1-3-0-documentation"></a>

#### Documentation

- **The docs spell out the formatting principles and normalizations**, and `README.rst` comes from them.
  ([#144](https://github.com/tox-dev/toml-fmt/pull/144), [#145](https://github.com/tox-dev/toml-fmt/pull/145)) by
  [@gaborbernat](https://github.com/gaborbernat).

### 1.2.2 (2026-01-07)

<a name="release-tox-toml-fmt-1-2-2"></a>

<a name="release-tox-toml-fmt-1-2-2-fixed"></a>

#### Fixed

- **A version written in parentheses reads.** `(>=1.0)` was rejected, so a requirement written that way came back
  unformatted. ([#122](https://github.com/tox-dev/toml-fmt/pull/122)) by [@jamesbursa](https://github.com/jamesbursa).

### 1.2.1 (2025-11-12)

<a name="release-tox-toml-fmt-1-2-1"></a>

<a name="release-tox-toml-fmt-1-2-1-fixed"></a>

#### Fixed

- **A version suffix reads only as far as the specification allows.** Parsing ran past the end of a version and took
  characters that belong to what follows it. ([#101](https://github.com/tox-dev/toml-fmt/pull/101)) by
  [@gaborbernat](https://github.com/gaborbernat).

<a name="release-tox-toml-fmt-1-2-1-documentation"></a>

#### Documentation

- **The tool names and the links between pages are correct.** ([#105](https://github.com/tox-dev/toml-fmt/pull/105),
  [#106](https://github.com/tox-dev/toml-fmt/pull/106)) by [@gaborbernat](https://github.com/gaborbernat) and
  [@jorisboeye](https://github.com/jorisboeye).

### 1.2.0 (2025-10-08)

<a name="release-tox-toml-fmt-1-2-0"></a>

<a name="release-tox-toml-fmt-1-2-0-added"></a>

#### Added

- **Python 3.14 wheels.** ([#96](https://github.com/tox-dev/toml-fmt/pull/96)) by
  [@gaborbernat](https://github.com/gaborbernat).
- **End support for Python 3.9.** ([#96](https://github.com/tox-dev/toml-fmt/pull/96)) by
  [@gaborbernat](https://github.com/gaborbernat).

<a name="release-tox-toml-fmt-1-2-0-fixed"></a>

#### Fixed

- **A pre-release label reads the way PEP 440 spells one.** `1.0rc1` and `1.0.rc.1` name the same release, and only one
  of them parsed. ([#89](https://github.com/tox-dev/toml-fmt/pull/89)) by [@adamchainz](https://github.com/adamchainz).

### 1.1.0 (2025-10-01)

<a name="release-tox-toml-fmt-1-1-0"></a>

<a name="release-tox-toml-fmt-1-1-0-changes"></a>

#### Changes

- **PEP 508 parsing moved in-tree.** The upstream crate was deprecated, so requirement parsing no longer depends on it.
  ([#84](https://github.com/tox-dev/toml-fmt/pull/84)) by [@gaborbernat](https://github.com/gaborbernat).

<a name="release-tox-toml-fmt-1-1-0-fixed"></a>

#### Fixed

- **A sorted table keeps the empty lines the file wrote around it.** Sorting collapsed the groups an author had spaced
  apart. ([#63](https://github.com/tox-dev/toml-fmt/pull/63)) by [@ddeepwell](https://github.com/ddeepwell).
- **Every part of a dotted key counts.** A key such as `set_env.PATH` was read as far as the first segment, so the rest
  decided nothing. ([#31](https://github.com/tox-dev/toml-fmt/pull/31)) by [@adamchainz](https://github.com/adamchainz).

### 1.0.0 (2024-10-31)

<a name="release-tox-toml-fmt-1-0-0"></a>

<a name="release-tox-toml-fmt-1-0-0-added"></a>

#### Added

- **A formatter for `tox.toml`.** It orders the root keys and each environment's keys, normalizes and sorts
  requirements, and lays the file out to a configured width. ([#1](https://github.com/tox-dev/toml-fmt/pull/1),
  [#18](https://github.com/tox-dev/toml-fmt/pull/18)) by [@gaborbernat](https://github.com/gaborbernat).
- **PEP 735 `dependency-groups` support.** ([#16](https://github.com/tox-dev/toml-fmt/pull/16)) by
  [@browniebroke](https://github.com/browniebroke) and [@gaborbernat](https://github.com/gaborbernat).

## toml-fmt-common

### 1.3.5 (2026-05-29)

<a name="release-toml-fmt-common-1-3-5"></a>

<a name="release-toml-fmt-common-1-3-5-fixed"></a>

#### Fixed

- **A consumer that registers its own formatting flags no longer gets duplicates.**
  ([#362](https://github.com/tox-dev/toml-fmt/pull/362)) by [@gaborbernat](https://github.com/gaborbernat).

### 1.3.4 (2026-05-29)

<a name="release-toml-fmt-common-1-3-4"></a>

<a name="release-toml-fmt-common-1-3-4-fixed"></a>

#### Fixed

- **`_build_cli` is back for consumers that called it.** Renaming it broke formatters built on this library.
  ([#361](https://github.com/tox-dev/toml-fmt/pull/361)) by [@gaborbernat](https://github.com/gaborbernat).

### 1.3.3 (2026-05-28)

<a name="release-toml-fmt-common-1-3-3"></a>
