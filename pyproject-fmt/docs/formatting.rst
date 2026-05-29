Formatting Rules
================

``pyproject-fmt`` is an opinionated formatter, much like `black <https://github.com/psf/black>`_ is for Python code.
The tool intentionally provides minimal configuration options because the goal is to establish a single standard format
that all ``pyproject.toml`` files follow.

**Benefits of this approach:**

- Less time configuring tools
- Smaller diffs when committing changes
- Easier code reviews since formatting is never a question

While a few key options exist (``column_width``, ``indent``, ``table_format``, ``sub_table_spacing``,
``separate_root_table``), the tool does not expose dozens of toggles. You get what the maintainers have chosen to be the
right balance of readability, consistency, and usability. The ``column_width`` setting controls when arrays are split
into multiple lines and when string values are wrapped using line continuations.

General Formatting
------------------

These rules apply uniformly across the entire ``pyproject.toml`` file.

Table Ordering
~~~~~~~~~~~~~~

Tables are reordered into a consistent structure:

1. ``[build-system]``
2. ``[project]``
3. ``[dependency-groups]``
4. ``[tool.*]`` sections in the order:

   1. Build backends: ``poetry``, ``poetry-dynamic-versioning``, ``pdm``, ``setuptools``, ``distutils``,
      ``setuptools_scm``, ``hatch``, ``flit``, ``scikit-build``, ``meson-python``, ``maturin``, ``pixi``,
      ``whey``, ``py-build-cmake``, ``sphinx-theme-builder``, ``uv``
   2. Builders: ``cibuildwheel``, ``nuitka``
   3. Linters/formatters: ``autopep8``, ``black``, ``ruff``, ``isort``, ``flake8``, ``pycln``, ``nbqa``,
      ``pylint``, ``repo-review``, ``codespell``, ``docformatter``, ``pydoclint``, ``tomlsort``,
      ``check-manifest``, ``check-sdist``, ``check-wheel-contents``, ``deptry``, ``pyproject-fmt``, ``typos``,
      ``bandit``
   4. Type checkers: ``mypy``, ``pyrefly``, ``pyright``, ``ty``, ``django-stubs``
   5. Testing: ``pytest``, ``pytest_env``, ``pytest-enabler``, ``coverage``
   6. Task runners: ``doit``, ``spin``, ``tox``
   7. Release tools: ``bumpversion``, ``jupyter-releaser``, ``tbump``, ``towncrier``, ``vendoring``
   8. Any other ``tool.*`` in alphabetical order

5. Any other tables (alphabetically)

String Quotes
~~~~~~~~~~~~~

All strings use double quotes by default. Single quotes are only used when the value contains double quotes:

.. code-block:: toml

    # Before
    name = 'my-package'
    description = "He said \"hello\""

    # After
    name = "my-package"
    description = 'He said "hello"'

Key Quotes
~~~~~~~~~~

TOML keys are normalized to the simplest valid form. Keys that are valid bare keys (containing only
``A-Za-z0-9_-``) have redundant quotes stripped. Single-quoted (literal) keys that require quoting are
converted to double-quoted (basic) strings with proper escaping. This applies to all keys: table headers,
key-value pairs, and inline table keys:

.. code-block:: toml

    # Before
    [tool."ruff"]
    "line-length" = 120
    lint.per-file-ignores.'tests/*' = ["S101"]

    # After
    [tool.ruff]
    line-length = 120
    lint.per-file-ignores."tests/*" = ["S101"]

Backslashes and double quotes within literal keys are escaped during conversion:

.. code-block:: toml

    # Before
    lint.per-file-ignores.'path\to\file' = ["E501"]

    # After
    lint.per-file-ignores."path\\to\\file" = ["E501"]

Array Formatting
~~~~~~~~~~~~~~~~

Arrays are formatted based on line length, trailing comma presence, and comments:

.. code-block:: toml

    # Short arrays stay on one line
    keywords = ["python", "toml"]

    # Long arrays that exceed column_width are expanded and get a trailing comma
    dependencies = [
        "requests>=2.28",
        "click>=8.0",
    ]

    # Trailing commas signal intent to keep multiline format
    classifiers = [
        "Development Status :: 4 - Beta",
    ]

    # Arrays with comments are always multiline
    lint.ignore = [
        "E501",  # Line too long
        "E701",
    ]

**Multiline formatting rules:**

An array becomes multiline when any of these conditions are met:

1. **Trailing comma present** - A trailing comma signals intent to keep multiline format
2. **Exceeds column width** - Arrays longer than ``column_width`` are expanded (and get a trailing comma added)
3. **Contains comments** - Arrays with inline or leading comments are always multiline

String Wrapping
~~~~~~~~~~~~~~~

Strings that exceed ``column_width`` (including the key name and ``" = "`` prefix) are wrapped into multi-line
triple-quoted strings using line continuations:

.. code-block:: toml

    # Before (exceeds column_width)
    description = "A very long description that goes beyond the configured column width limit"

    # After
    description = """\
      A very long description that goes beyond the \
      configured column width limit\
      """

Wrapping prefers breaking at spaces and at ``" :: "`` separators (common in Python classifiers). Strings inside inline
tables are never wrapped. Strings that contain actual newlines are preserved as multi-line strings without adding line
continuations. Use ``skip_wrap_for_keys`` to prevent wrapping for specific keys.

Table Formatting
~~~~~~~~~~~~~~~~

Sub-tables can be formatted in two styles controlled by ``table_format``:

**Short format** (collapsed to dotted keys):

.. code-block:: toml

    [project]
    urls.homepage = "https://example.com"
    urls.repository = "https://github.com/example/project"

**Long format** (expanded to table headers):

.. code-block:: toml

    [project.urls]
    homepage = "https://example.com"
    repository = "https://github.com/example/project"

**Table spacing:**

By default, different table groups (e.g. ``[project]`` and ``[tool.ruff]``) are separated by a blank line, while
sub-tables within the same group (e.g. ``[tool.ruff]`` and ``[tool.ruff.lint]``) are kept compact with no blank line
between them. You can control this with ``sub_table_spacing`` and ``separate_root_table``. Each option takes a string of
``\n`` characters where each ``\n`` adds one blank line. For example, setting ``sub_table_spacing = "\n"`` adds a blank
line between sub-tables:

.. code-block:: toml

    [tool.ruff]
    line-length = 120

    [tool.ruff.lint]
    select = ["E", "W"]

See :doc:`configuration` for how to control this behavior.

Comment Preservation
~~~~~~~~~~~~~~~~~~~~

All comments are preserved during formatting:

- **Inline comments** - Comments after a value on the same line stay with that value
- **Leading comments** - Comments on the line before an entry stay with the entry below
- **Block comments** - Multi-line comment blocks are preserved

**Inline comment alignment:**

Inline comments within arrays are aligned independently per array, based on that array's longest value:

.. code-block:: toml

    # Before - comments at inconsistent positions
    lint.ignore = [
      "COM812", # Conflict with formatter
      "CPY", # No copyright statements
      "ISC001",   # Another rule
    ]

    # After - comments align to longest value in this array
    lint.ignore = [
      "COM812",  # Conflict with formatter
      "CPY",     # No copyright statements
      "ISC001",  # Another rule
    ]

Table-Specific Handling
-----------------------

Beyond general formatting, each table has specific key ordering and value normalization rules.

``[build-system]``
~~~~~~~~~~~~~~~~~~

**Key ordering:** ``build-backend`` → ``requires`` → ``backend-path``

**Value normalization:**

- ``requires``: Dependencies normalized per PEP 508 and sorted alphabetically by package name
- ``backend-path``: Entries sorted alphabetically

.. code-block:: toml

    # Before
    [build-system]
    requires = ["setuptools >= 45", "wheel"]
    build-backend = "setuptools.build_meta"

    # After
    [build-system]
    build-backend = "setuptools.build_meta"
    requires = ["setuptools>=45", "wheel"]

``[project]``
~~~~~~~~~~~~~

**Key ordering:**

Keys are reordered in this sequence: ``name`` → ``version`` → ``import-names`` → ``import-namespaces`` →
``description`` → ``readme`` → ``keywords`` → ``license`` → ``license-files`` → ``maintainers`` → ``authors`` →
``requires-python`` → ``classifiers`` → ``dynamic`` → ``dependencies`` → ``optional-dependencies`` → ``urls`` →
``scripts`` → ``gui-scripts`` → ``entry-points``

**Field normalizations:**

``name``
    Converted to canonical format (lowercase with hyphens): ``My_Package`` → ``my-package``

``description``
    Whitespace normalized: multiple spaces collapsed, consistent spacing after periods.

``license``
    License expression operators (``and``, ``or``, ``with``) uppercased: ``MIT or Apache-2.0`` → ``MIT OR Apache-2.0``

``requires-python``
    Whitespace removed: ``>= 3.9`` → ``>=3.9``

``keywords``
    Deduplicated (case-insensitive) and sorted alphabetically.

``dynamic``
    Sorted alphabetically.

``import-names`` / ``import-namespaces``
    Semicolon spacing normalized (``foo;bar`` → ``foo; bar``), entries sorted alphabetically.

``classifiers``
    Deduplicated and sorted alphabetically.

``authors`` / ``maintainers``
    Sorted by name, then email. Keys within each entry ordered: ``name`` → ``email``.

**Dependency normalization:**

All dependency arrays (``dependencies``, ``optional-dependencies.*``) are:

- Normalized per PEP 508: spaces removed, redundant ``.0`` suffixes stripped (unless ``keep_full_version = true``)
- Sorted alphabetically by canonical package name

.. code-block:: toml

    # Before
    dependencies = ["requests >= 2.0.0", "click~=8.0"]

    # After
    dependencies = ["click>=8", "requests>=2"]

**Optional dependencies extra names:**

Extra names are normalized to lowercase with hyphens:

.. code-block:: toml

    # Before
    [project.optional-dependencies]
    Dev_Tools = ["pytest"]

    # After
    [project.optional-dependencies]
    dev-tools = ["pytest"]

**Python version classifiers:**

Classifiers for Python versions are automatically generated based on ``requires-python`` and
``max_supported_python``. Disable with ``generate_python_version_classifiers = false``.

.. code-block:: toml

    # With requires-python = ">=3.10" and max_supported_python = "3.14"
    classifiers = [
        "Programming Language :: Python :: 3 :: Only",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Programming Language :: Python :: 3.13",
        "Programming Language :: Python :: 3.14",
    ]

**Entry points:**

Inline tables within ``entry-points`` are expanded to dotted keys:

.. code-block:: toml

    # Before
    entry-points.console_scripts = { mycli = "mypackage:main" }

    # After
    entry-points.console_scripts.mycli = "mypackage:main"

**Authors/maintainers formatting:**

Contact information can be formatted as inline tables or expanded array of tables:

.. code-block:: toml

    # Short format (inline)
    authors = [{ name = "Alice", email = "alice@example.com" }]

    # Long format (array of tables)
    [[project.authors]]
    name = "Alice"
    email = "alice@example.com"

Controlled by ``table_format``, ``expand_tables``, and ``collapse_tables``.

``[dependency-groups]``
~~~~~~~~~~~~~~~~~~~~~~~

**Key ordering:** ``dev`` → ``test`` → ``type`` → ``docs`` → others alphabetically

**Value normalization:**

- All dependencies normalized per PEP 508
- Sorted: regular dependencies first, then ``include-group`` entries

.. code-block:: toml

    # Before
    [dependency-groups]
    dev = [{ include-group = "test" }, "ruff>=0.4", "mypy>=1"]

    # After
    [dependency-groups]
    dev = ["mypy>=1", "ruff>=0.4", { include-group = "test" }]

``[tool.ruff]``
~~~~~~~~~~~~~~~

**Key ordering:**

Keys are reordered in a logical sequence:

1. Global settings: ``required-version`` → ``extend`` → ``target-version`` → ``line-length`` → ``indent-width`` →
   ``tab-size``
2. Path settings: ``builtins`` → ``namespace-packages`` → ``src`` → ``include`` → ``extend-include`` → ``exclude`` →
   ``extend-exclude`` → ``force-exclude`` → ``respect-gitignore``
3. Behavior flags: ``preview`` → ``fix`` → ``unsafe-fixes`` → ``fix-only`` → ``show-fixes`` → ``show-source``
4. Output settings: ``output-format`` → ``cache-dir``
5. ``format.*`` keys
6. ``lint.*`` keys: ``select`` → ``extend-select`` → ``ignore`` → ``extend-ignore`` → ``per-file-ignores`` →
   ``fixable`` → ``unfixable`` → plugin configurations

**Sorted arrays:**

Arrays are sorted alphabetically using natural ordering (``RUF1`` < ``RUF9`` < ``RUF10``):

.. code-block:: toml

    # These arrays are sorted:
    lint.select = ["E", "F", "I", "RUF"]
    lint.ignore = ["E501", "E701"]

    # Per-file-ignores values are also sorted:
    lint.per-file-ignores."tests/*.py" = ["D103", "S101"]

**Sorted array keys:**

Top-level:
  ``exclude``, ``extend-exclude``, ``include``, ``extend-include``, ``builtins``, ``namespace-packages``, ``src``

Format:
  ``format.exclude``

Lint:
  ``select``, ``extend-select``, ``ignore``, ``extend-ignore``, ``fixable``, ``extend-fixable``, ``unfixable``,
  ``extend-safe-fixes``, ``extend-unsafe-fixes``, ``external``, ``task-tags``, ``exclude``, ``typing-modules``,
  ``allowed-confusables``, ``logger-objects``

Per-file patterns:
  ``lint.per-file-ignores.*``, ``lint.extend-per-file-ignores.*``

Plugin arrays:
  ``lint.flake8-bandit.hardcoded-tmp-directory``, ``lint.flake8-bandit.hardcoded-tmp-directory-extend``,
  ``lint.flake8-boolean-trap.extend-allowed-calls``, ``lint.flake8-bugbear.extend-immutable-calls``,
  ``lint.flake8-builtins.builtins-ignorelist``, ``lint.flake8-gettext.extend-function-names``,
  ``lint.flake8-gettext.function-names``, ``lint.flake8-import-conventions.banned-from``,
  ``lint.flake8-pytest-style.raises-extend-require-match-for``, ``lint.flake8-pytest-style.raises-require-match-for``,
  ``lint.flake8-self.extend-ignore-names``, ``lint.flake8-self.ignore-names``,
  ``lint.flake8-tidy-imports.banned-module-level-imports``, ``lint.flake8-type-checking.exempt-modules``,
  ``lint.flake8-type-checking.runtime-evaluated-base-classes``,
  ``lint.flake8-type-checking.runtime-evaluated-decorators``, ``lint.isort.constants``,
  ``lint.isort.default-section``, ``lint.isort.extra-standard-library``, ``lint.isort.forced-separate``,
  ``lint.isort.no-lines-before``, ``lint.isort.required-imports``, ``lint.isort.single-line-exclusions``,
  ``lint.isort.variables``, ``lint.pep8-naming.classmethod-decorators``, ``lint.pep8-naming.extend-ignore-names``,
  ``lint.pep8-naming.ignore-names``, ``lint.pep8-naming.staticmethod-decorators``,
  ``lint.pydocstyle.ignore-decorators``, ``lint.pydocstyle.property-decorators``, ``lint.pyflakes.extend-generics``,
  ``lint.pylint.allow-dunder-method-names``, ``lint.pylint.allow-magic-value-types``

``[tool.pixi]``
~~~~~~~~~~~~~~~

**Key ordering:**

Keys are grouped by functionality:

1. Workspace metadata: ``workspace.name`` → ``workspace.version`` → ``workspace.description`` →
   ``workspace.authors`` → ``workspace.license`` → ``workspace.license-file`` → ``workspace.readme`` →
   ``workspace.homepage`` → ``workspace.repository`` → ``workspace.documentation``
2. Workspace configuration: ``workspace.channels`` → ``workspace.platforms`` → ``workspace.channel-priority`` →
   ``workspace.solve-strategy`` → ``workspace.conda-pypi-map`` → ``workspace.requires-pixi`` →
   ``workspace.exclude-newer`` → ``workspace.preview`` → ``workspace.build-variants`` →
   ``workspace.build-variants-files``
3. Dependencies: ``dependencies`` → ``host-dependencies`` → ``build-dependencies`` → ``run-dependencies`` →
   ``constraints`` → ``pypi-dependencies`` → ``pypi-options``
4. Development: ``dev``
5. Environment setup: ``system-requirements`` → ``activation`` → ``tasks``
6. Targeting: ``target`` → ``feature`` → ``environments``
7. Package build: ``package``

**Sorted arrays:**

``workspace.channels``, ``workspace.platforms``, ``workspace.preview``, ``workspace.build-variants-files``

``[tool.uv]``
~~~~~~~~~~~~~

**Key ordering:**

Keys are grouped by functionality:

1. Version & Python: ``required-version`` → ``python-preference`` → ``python-downloads``
2. Dependencies: ``dev-dependencies`` → ``default-groups`` → ``dependency-groups`` → ``constraint-dependencies`` →
   ``override-dependencies`` → ``exclude-dependencies`` → ``dependency-metadata``
3. Sources & indexes: ``sources`` → ``index`` → ``index-url`` → ``extra-index-url`` → ``find-links`` → ``no-index`` →
   ``index-strategy`` → ``keyring-provider``
4. Package handling: ``no-binary*`` → ``no-build*`` → ``no-sources*`` → ``reinstall*`` → ``upgrade*``
5. Resolution: ``resolution`` → ``prerelease`` → ``fork-strategy`` → ``environments`` → ``required-environments`` →
   ``exclude-newer*``
6. Build & Install: ``compile-bytecode`` → ``link-mode`` → ``config-settings*`` → ``extra-build-*`` →
   ``concurrent-builds`` → ``concurrent-downloads`` → ``concurrent-installs``
7. Network & Security: ``allow-insecure-host`` → ``native-tls`` → ``offline`` → ``no-cache`` → ``cache-dir`` →
   ``http-proxy`` → ``https-proxy`` → ``no-proxy``
8. Publishing: ``publish-url`` → ``check-url`` → ``trusted-publishing``
9. Python management: ``python-install-mirror`` → ``pypy-install-mirror`` → ``python-downloads-json-url``
10. Workspace & Project: ``managed`` → ``package`` → ``workspace`` → ``conflicts`` → ``cache-keys`` → ``build-backend``
11. Other: ``pip`` → ``preview`` → ``torch-backend``

**Sorted arrays:**

Package name arrays (sorted alphabetically):
  ``constraint-dependencies``, ``override-dependencies``, ``dev-dependencies``, ``exclude-dependencies``,
  ``no-binary-package``, ``no-build-package``, ``no-build-isolation-package``, ``no-sources-package``,
  ``reinstall-package``, ``upgrade-package``

Other arrays:
  ``environments``, ``required-environments``, ``allow-insecure-host``, ``no-proxy``, ``workspace.members``,
  ``workspace.exclude``

**Sources table:**

The ``sources`` table entries are sorted alphabetically by package name:

.. code-block:: toml

    # Before
    [tool.uv.sources]
    zebra = { git = "..." }
    alpha = { path = "..." }

    # After
    [tool.uv.sources]
    alpha = { path = "..." }
    zebra = { git = "..." }

**pip subsection:**

The ``[tool.uv.pip]`` subsection follows similar formatting rules, with arrays like ``extra``, ``no-binary-package``,
``no-build-package``, ``reinstall-package``, and ``upgrade-package`` sorted alphabetically.

``[tool.coverage]``
~~~~~~~~~~~~~~~~~~~

**Key ordering:**

Keys are reordered to follow coverage.py's workflow phases:

1. **Run phase** (``run.*``): Data collection settings

   - Source selection: ``source`` → ``source_pkgs`` → ``source_dirs``
   - File filtering: ``include`` → ``omit``
   - Measurement: ``branch`` → ``cover_pylib`` → ``timid``
   - Execution context: ``command_line`` → ``concurrency`` → ``context`` → ``dynamic_context``
   - Data management: ``data_file`` → ``parallel`` → ``relative_files``
   - Extensions: ``plugins``
   - Debugging: ``debug`` → ``debug_file`` → ``disable_warnings``
   - Other: ``core`` → ``patch`` → ``sigterm``

2. **Paths** (``paths.*``): Path mapping between source locations

3. **Report phase** (``report.*``): General reporting

   - Thresholds: ``fail_under`` → ``precision``
   - File filtering: ``include`` → ``omit`` → ``include_namespace_packages``
   - Line exclusion: ``exclude_lines`` → ``exclude_also``
   - Partial branches: ``partial_branches`` → ``partial_also``
   - Output control: ``skip_covered`` → ``skip_empty`` → ``show_missing``
   - Formatting: ``format`` → ``sort``
   - Error handling: ``ignore_errors``

4. **Output formats** (after report):

   - ``html.*``: ``directory`` → ``title`` → ``extra_css`` → ``show_contexts`` → ``skip_covered`` → ``skip_empty``
   - ``json.*``: ``output`` → ``pretty_print`` → ``show_contexts``
   - ``lcov.*``: ``output`` → ``line_checksums``
   - ``xml.*``: ``output`` → ``package_depth``

**Grouping principle:**

Related options are grouped together:

- File selection: ``include``/``omit`` are adjacent
- Exclusion patterns: ``exclude_lines``/``exclude_also`` are adjacent
- Partial branches: ``partial_branches``/``partial_also`` are adjacent
- Skip options: ``skip_covered``/``skip_empty`` are adjacent

**Sorted arrays:**

Run phase:
  ``source``, ``source_pkgs``, ``source_dirs``, ``include``, ``omit``, ``concurrency``, ``plugins``, ``debug``,
  ``disable_warnings``

Report phase:
  ``include``, ``omit``, ``exclude_lines``, ``exclude_also``, ``partial_branches``, ``partial_also``

.. code-block:: toml

    # Before (alphabetical)
    [tool.coverage]
    report.exclude_also = ["if TYPE_CHECKING:"]
    report.omit = ["tests/*"]
    run.branch = true
    run.omit = ["tests/*"]

    # After (workflow order with groupings)
    [tool.coverage]
    run.branch = true
    run.omit = ["tests/*"]
    report.omit = ["tests/*"]
    report.exclude_also = ["if TYPE_CHECKING:"]

``[tool.commitizen]``
~~~~~~~~~~~~~~~~~~~~~

Top-level ordering: rule selection (``name``, ``schema``, ``schema_pattern``, ``allowed_prefixes``) → version
source (``version``, ``version_scheme``, ``version_provider``, ``version_files``) → bump behavior → tag/sign →
changelog → hooks (``pre_bump_hooks``, ``post_bump_hooks``) → ``customize``.

**Sorted arrays:** ``version_files``, ``allowed_prefixes``, ``extras``, ``extra_files``.
``[tool.poetry]``
~~~~~~~~~~~~~~~~~

Covers both Poetry 1.x (legacy metadata under ``[tool.poetry]``) and Poetry 2.x (metadata moved to standard
``[project]``; Poetry-specific keys still under ``[tool.poetry]``).

**Top-level key ordering:**

1. Identity: ``name`` → ``version`` → ``description`` → ``package-mode``
2. License & authorship: ``license`` → ``authors`` → ``maintainers``
3. Documentation: ``readme`` → ``homepage`` → ``repository`` → ``documentation``
4. Discovery: ``keywords`` → ``classifiers``
5. Packaging contents: ``packages`` → ``include`` → ``exclude`` → ``build``
6. Dependencies (sub-tables): ``dependencies`` → ``dev-dependencies`` → ``group`` → ``extras``
7. Entry points / distribution: ``scripts`` → ``plugins`` → ``urls`` → ``source``
8. Poetry runtime constraints: ``requires-poetry`` → ``requires-plugins`` → ``build-constraints``

**Sub-table key ordering:**

``[tool.poetry.dependencies]`` / ``[tool.poetry.dev-dependencies]`` / per-group dependencies
    ``python`` first (interpreter constraint), all other package names alphabetized.

``[tool.poetry.group.<name>]``
    ``optional`` → ``include-groups`` → ``dependencies``.

``[tool.poetry.extras]``, ``[tool.poetry.scripts]``, ``[tool.poetry.urls]``, ``[tool.poetry.plugins.*]``, ``[tool.poetry.requires-plugins]``, ``[tool.poetry.build-constraints]``
    Keys alphabetized.

``[tool.poetry.build]``
    ``script`` → ``generate-setup-file``.

``[[tool.poetry.source]]``
    Each entry's keys ordered ``name`` → ``url`` → ``priority`` → ``links`` → ``indexed``, with the deprecated
    ``default`` and ``secondary`` keys placed last. Array order itself is preserved (priority ordering is
    semantically significant).

**Sorted arrays:**

- ``keywords``, ``classifiers``: deduplicated (case-insensitive) and sorted alphabetically.
- ``exclude``: sorted alphabetically.
- ``[tool.poetry.extras]`` values (each ``extras.<name>``): sorted alphabetically.
- ``[tool.poetry.group.<name>.include-groups]``: sorted alphabetically.
- Per-dependency ``extras`` arrays (in ``dependencies``, ``dev-dependencies``, per-group dependencies,
  ``requires-plugins``, ``build-constraints``): sorted alphabetically.

Arrays whose order carries semantic meaning are preserved as written: ``authors``, ``maintainers``, ``packages``,
``include``, ``readme`` (when an array), multi-constraint dependency arrays, and ``[[tool.poetry.source]]`` entries.

**Inline-table key ordering:**

When a Poetry-specific inline table is detected (via discriminator keys unique to Poetry's schema), its keys are
reordered:

- Sources (``{ priority = ... }``, ``{ secondary = ... }``, ``{ links = ... }``, ``{ indexed = ... }``):
  ``name`` → ``url`` → ``priority`` → ``links`` → ``indexed`` → ``default`` → ``secondary``.
- Git dependencies (``{ git = ... }``):
  ``git`` → ``branch`` → ``tag`` → ``rev`` → ``subdirectory`` → ``python`` → ``platform`` → ``markers`` →
  ``allow-prereleases`` → ``allows-prereleases`` → ``optional`` → ``extras`` → ``develop``.
- Path dependencies (``{ path = ... }``):
  ``path`` → ``develop`` → ``subdirectory`` → ``python`` → ``platform`` → ``markers`` → ``optional`` → ``extras``.
- File dependencies (``{ file = ... }``):
  ``file`` → ``subdirectory`` → ``python`` → ``platform`` → ``markers`` → ``optional`` → ``extras``.

Inline tables that don't match any Poetry-specific schema (for example ``[[project.authors]]`` inline form
``{ name = "...", email = "..." }``) are left untouched.

.. code-block:: toml

    # Before
    [[tool.poetry.source]]
    priority = "primary"
    url = "https://pypi.example.com/simple"
    name = "private"

    [tool.poetry.dependencies]
    zebra = "^1.0"
    python = "^3.11"
    foo = { branch = "main", git = "https://github.com/example/foo" }

    # After
    [tool.poetry]
    dependencies.python = "^3.11"
    dependencies.foo = { git = "https://github.com/example/foo", branch = "main" }
    dependencies.zebra = "^1.0"
    source = [ { name = "private", url = "https://pypi.example.com/simple", priority = "primary" } ]
``[tool.mypy]``
~~~~~~~~~~~~~~~

Covers all documented mypy options plus the ``[[tool.mypy.overrides]]`` array of tables. Keys are reordered to match
the section structure of the official mypy configuration reference.

**Top-level key ordering** (sectioned):

1. Import discovery: ``mypy_path`` → ``files`` → ``modules`` → ``packages`` → ``exclude`` → ``exclude_gitignore`` →
   ``namespace_packages`` → ``explicit_package_bases`` → ``ignore_missing_imports`` → ``follow_untyped_imports`` →
   ``follow_imports`` → ``follow_imports_for_stubs`` → ``python_executable`` → ``no_site_packages`` →
   ``no_silence_site_packages``
2. Platform configuration: ``python_version`` → ``platform`` → ``always_true`` → ``always_false``
3. Disallow dynamic typing: ``disallow_any_unimported`` → ``disallow_any_expr`` → ``disallow_any_decorated`` →
   ``disallow_any_explicit`` → ``disallow_any_generics`` → ``disallow_subclassing_any``
4. Untyped definitions and calls: ``disallow_untyped_calls`` → ``untyped_calls_exclude`` → ``disallow_untyped_defs``
   → ``disallow_incomplete_defs`` → ``check_untyped_defs`` → ``disallow_untyped_decorators``
5. None and Optional: ``implicit_optional`` → ``strict_optional``
6. Configuring warnings: ``warn_redundant_casts`` → ``warn_unused_ignores`` → ``warn_no_return`` →
   ``warn_return_any`` → ``warn_unreachable`` → ``deprecated_calls_exclude``
7. Suppressing errors: ``ignore_errors``
8. Miscellaneous strictness: ``allow_untyped_globals`` → ``allow_redefinition`` → ``local_partial_types`` →
   ``disable_error_code`` → ``enable_error_code`` → ``extra_checks`` → ``implicit_reexport`` →
   ``strict_equality`` → ``strict_bytes`` → ``strict``
9. Configuring error messages: ``show_error_context`` → ``show_column_numbers`` → ``show_error_end`` →
   ``hide_error_codes`` → ``show_error_code_links`` → ``pretty`` → ``color_output`` → ``error_summary`` →
   ``show_absolute_path``
10. Incremental mode: ``incremental`` → ``cache_dir`` → ``sqlite_cache`` → ``cache_fine_grained`` →
    ``skip_version_check`` → ``skip_cache_mtime_checks``
11. Advanced options: ``plugins`` → ``pdb`` → ``show_traceback`` → ``raise_exceptions`` →
    ``custom_typing_module`` → ``custom_typeshed_dir`` → ``warn_incomplete_stub`` → ``native_parser``
12. Report generation: ``any_exprs_report`` → ``cobertura_xml_report`` → ``html_report`` → ``linecount_report`` →
    ``linecoverage_report`` → ``lineprecision_report`` → ``txt_report`` → ``xml_report`` → ``xslt_html_report`` →
    ``xslt_txt_report``
13. Miscellaneous: ``junit_xml`` → ``junit_format`` → ``scripts_are_modules`` → ``warn_unused_configs`` →
    ``verbosity``
14. ``overrides`` last.

**``[[tool.mypy.overrides]]`` entry key ordering:**

``module`` first (required), then per-module overridable keys in the same logical grouping as the parent table
(import behavior, platform markers, disallow dynamic typing, untyped defs/calls, optional handling, warnings,
suppression, miscellaneous strictness).

**Sorted arrays:**

- Top-level: ``files``, ``modules``, ``packages``, ``exclude``, ``always_true``, ``always_false``,
  ``untyped_calls_exclude``, ``deprecated_calls_exclude``, ``disable_error_code``, ``enable_error_code``.
- Inside overrides entries: ``module`` (when an array of patterns), ``always_true``, ``always_false``,
  ``disable_error_code``, ``enable_error_code``.

``plugins`` and ``mypy_path`` are deliberately preserved as written: plugins run in declared order and reordering
changes behavior; ``mypy_path`` is a search path with priority semantics.

**Inline-table handling:**

When ``[[tool.mypy.overrides]]`` collapses to ``overrides = [{...}, {...}]`` under the default ``table_format =
"short"``, key order inside each entry is normalized via discriminators unique to mypy
(``disable_error_code`` / ``enable_error_code`` / ``ignore_missing_imports`` / ``follow_untyped_imports`` /
``ignore_errors`` / ``warn_unused_ignores`` / ``disallow_untyped_defs`` / ``check_untyped_defs``). The arrays inside
each inline entry are sorted in place, so ``disable_error_code = [...]`` is alphabetized whether the override is
expanded or collapsed.

.. code-block:: toml

    # Before
    [[tool.mypy.overrides]]
    ignore_missing_imports = true
    disable_error_code = ["import-untyped", "attr-defined"]
    module = "third_party.*"

    # After
    [tool.mypy]
    overrides = [
      { module = "third_party.*", ignore_missing_imports = true, disable_error_code = [ "attr-defined", "import-untyped" ] },
    ]
``[tool.setuptools]`` and ``[tool.setuptools_scm]``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Covers the setuptools build backend and the version-from-SCM plugin.

**``[tool.setuptools]`` top-level key ordering** (grouped):

1. Packaging discovery: ``py-modules`` → ``packages.find.*`` / ``packages.find-namespace.*`` → ``packages`` →
   ``package-dir``
2. Package data: ``include-package-data`` → ``package-data`` → ``exclude-package-data``
3. Dynamic metadata: ``dynamic``
4. Extensions / build customization: ``ext-modules`` → ``cmdclass``
5. Distribution metadata: ``platforms`` → ``provides`` → ``obsoletes`` → ``license-files``
6. Data files: ``data-files``
7. Deprecated / obsolete (pushed last): ``script-files`` → ``namespace-packages`` → ``zip-safe`` →
   ``eager-resources`` → ``dependency-links``

**``[tool.setuptools.packages.find]`` / ``[tool.setuptools.packages.find-namespace]`` inner ordering:**

``where`` → ``include`` → ``exclude`` → ``namespaces``.

**``[tool.setuptools.package-data]`` / ``[tool.setuptools.exclude-package-data]`` / ``[tool.setuptools.data-files]``
ordering:**

The catch-all ``"*"`` pattern always goes first, then the other package patterns alphabetically. Each value (an
array of glob patterns) is sorted alphabetically.

**``[tool.setuptools.dynamic]`` ordering:**

Field names alphabetized. Inline-table directives (e.g. ``version = { attr = "pkg.__version__" }`` or
``readme = { file = "README.md", content-type = "text/markdown" }``) get their keys ordered ``attr`` → ``file`` →
``content-type``.

**Sorted arrays:**

- ``py-modules``, ``platforms``, ``provides``, ``obsoletes``, ``script-files``, ``namespace-packages``,
  ``eager-resources``: alphabetized.
- ``packages.find.include`` / ``packages.find.exclude`` / ``packages.find-namespace.*``: alphabetized.
- Values inside ``package-data`` / ``exclude-package-data`` / ``data-files`` tables: alphabetized.

Arrays whose order is meaningful are preserved as written: ``packages`` (literal list — first match wins),
``license-files`` (PEP 639 concatenation order), and everything under ``[[tool.setuptools.ext-modules]]`` (compiler
and linker argv arrays).

**``[tool.setuptools_scm]`` key ordering** (grouped):

1. Version output: ``version_file`` → ``version_file_template``
2. Version computation: ``version_scheme`` → ``local_scheme`` → ``version_cls`` → ``normalize``
3. Root discovery: ``root`` → ``relative_to`` → ``fallback_root`` → ``parent`` → ``search_parent_directories`` →
   ``dist_name``
4. Tag / parse: ``tag_regex`` → ``parse`` → ``parentdir_prefix_version`` → ``fallback_version``
5. Nested SCM-specific tables: ``scm.git.pre_parse`` → ``scm.git.describe_command``
6. Deprecated (pushed last): ``git_describe_command`` (use ``scm.git.describe_command``) → ``write_to`` (use
   ``version_file``) → ``write_to_template`` (use ``version_file_template``) → ``version_class`` (use
   ``version_cls``) → ``template``

.. code-block:: toml

    # Before
    [tool.setuptools]
    zip-safe = false
    py-modules = ["foo", "bar"]
    packages = ["my_pkg"]

    [tool.setuptools.packages.find]
    namespaces = true
    where = ["src"]
    include = ["my_pkg*"]

    [tool.setuptools.dynamic]
    readme = { content-type = "text/markdown", file = "README.md" }

    # After
    [tool.setuptools]
    py-modules = [ "bar", "foo" ]
    packages.find.where = [ "src" ]
    packages.find.include = [ "my_pkg*" ]
    packages.find.namespaces = true
    packages = [ "my_pkg" ]
    dynamic.readme = { file = "README.md", content-type = "text/markdown" }
    zip-safe = false
``[tool.pytest.ini_options]``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Covers the pytest configuration block. Keys are grouped to follow the pytest reference: pytest itself →
discovery → CLI arguments → markers/parametrize → warnings → doctest → output → logging (capture / CLI / file)
→ JUnit XML → cache and tmp_path → assertion / faulthandler.

**Sorted arrays** (set/unordered semantics):

``testpaths``, ``norecursedirs``, ``collect_ignore``, ``collect_ignore_glob``, ``python_files``,
``python_classes``, ``python_functions``, ``markers``, ``filterwarnings``, ``doctest_optionflags``,
``usefixtures``, ``required_plugins``.

``addopts`` and ``pythonpath`` are deliberately preserved as written: ``addopts`` is CLI argv (order matters)
and ``pythonpath`` is a search path with priority semantics.

.. code-block:: toml

    # Before
    [tool.pytest.ini_options]
    log_cli_level = "INFO"
    markers = [ "slow: marks tests as slow", "fast: marks tests as fast" ]
    addopts = [ "--strict-markers", "-ra" ]
    testpaths = [ "tests" ]
    minversion = "8"

    # After
    [tool.pytest]
    ini_options.minversion = "8"
    ini_options.testpaths = [ "tests" ]
    ini_options.addopts = [ "--strict-markers", "-ra" ]
    ini_options.markers = [ "fast: marks tests as fast", "slow: marks tests as slow" ]
    ini_options.log_cli_level = "INFO"
``[tool.black]``
~~~~~~~~~~~~~~~~

Black's configuration is small but ubiquitous. Keys are ordered: ``required-version`` → ``target-version`` →
``line-length`` → ``include`` / ``extend-exclude`` / ``force-exclude`` / ``exclude`` → behavior flags
(``skip-string-normalization``, ``skip-magic-trailing-comma``, ``preview``, ``unstable``,
``enable-unstable-feature``, ``fast``, ``workers``) → output (``color``, ``verbose``, ``quiet``).

**Sorted arrays:**

- ``target-version``: alphabetized so ``py39`` precedes ``py310`` etc.
- ``enable-unstable-feature``: alphabetized.

The ``include`` / ``exclude`` family are regex strings, not arrays, so they're left as-is.
``[tool.hatch.*]``
~~~~~~~~~~~~~~~~~~

Hatch configuration spans many sub-tables. Keys at ``[tool.hatch]`` level (which after collapse appear as dotted
``version.*`` / ``build.*`` / ``metadata.*`` / ``envs.*`` / ``publish.*`` / ``workspace.*``) are ordered:

1. Version: ``version.source`` → ``version.path`` → ``version.pattern`` → ``version.expression`` →
   ``version.scheme`` → ``version.validate-bump`` → ``version.fallback-version`` → ``version.raw-options``.
2. Metadata: ``metadata.allow-direct-references`` → ``metadata.allow-ambiguous-features`` → ``metadata.hooks``.
3. Build: ``build.dev-mode-dirs`` → ``build.directory`` → ``build.sources`` → ``build.packages`` →
   ``build.include`` → ``build.exclude`` → ``build.force-include`` → ``build.artifacts`` → ``build.ignore-vcs`` →
   ``build.skip-excluded-dirs`` → ``build.reproducible`` → ``build.hooks`` → wheel target (``packages``,
   ``include``, ``exclude``, ``force-include``, ``artifacts``, ``hooks``, ``shared-data``, ``extra-metadata``,
   etc.) → sdist target (``include``, ``exclude``, ``force-include``, ``support-legacy``, ``strict-naming``).
4. Publish: ``publish.index.disable`` → ``publish.index.repos`` → ``publish.index``.
5. Workspace: ``workspace.members`` → ``workspace.exclude``.
6. Environments (``envs.<name>.*``): each environment's keys follow ``type`` → ``template`` → ``detached`` →
   ``description`` → ``platforms`` → ``python`` → ``path`` → ``installer`` → ``skip-install`` →
   ``system-packages`` → ``dev-mode`` → ``features`` → ``dependencies`` → ``extra-dependencies`` →
   ``extra-args`` → ``pre-install-commands`` → ``post-install-commands`` → ``env-include`` → ``env-exclude``
   → ``env-vars`` → ``scripts`` → ``matrix`` → ``matrix-name-format`` → ``overrides``.

**Sorted arrays:**

- Build: ``include``, ``exclude``, ``force-include``, ``artifacts``, ``packages``, ``sources``, ``dev-mode-dirs``,
  and the matching ``build.targets.wheel.*`` / ``build.targets.sdist.*`` arrays.
- Environments: per-env ``dependencies``, ``extra-dependencies``, ``features``, ``platforms``, ``env-include``,
  ``env-exclude``, ``pre-install-commands``, ``post-install-commands``.
- Workspace: ``members``, ``exclude``.

``scripts`` and ``env-vars`` sub-tables under each environment have their inner keys alphabetized. Build hook
order and matrix entry order are preserved as written (both carry semantic meaning).

Other Tables
~~~~~~~~~~~~

Any unrecognized tables are preserved and reordered according to standard table ordering rules. Keys within unknown
tables are not reordered or normalized.
