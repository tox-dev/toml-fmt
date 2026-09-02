pyproject-fmt
=============

``pyproject-fmt`` formats ``pyproject.toml`` files without discarding comments. It uses a small configuration surface
to keep results stable across projects. See the
`release history <https://github.com/tox-dev/toml-fmt/releases?q=pyproject-fmt>`_ for version changes.

Install
-------

The command requires Python 3.10 or later. An isolated tool environment avoids dependency conflicts with the target
project.


    .. code-block:: bash

        uv tool install pyproject-fmt
        pyproject-fmt --help

Pre-commit
----------

Add the hook to ``.pre-commit-config.yaml`` and set ``rev`` to the required release:

.. code-block:: yaml

    - repo: https://github.com/tox-dev/pyproject-fmt
      rev: ""
      hooks:
        - id: pyproject-fmt

See `pre-commit/pre-commit <https://github.com/pre-commit/pre-commit>`_ for installation and update commands.

Python API
----------

``run`` accepts command-line arguments and returns the process exit code:

.. code-block:: python

    from pyproject_fmt import run

    exit_code = run(["path/to/pyproject.toml"])

See the `configuration reference <https://pyproject-fmt.readthedocs.io/en/latest/configuration.html>`_ for settings and
the `formatting reference <https://pyproject-fmt.readthedocs.io/en/latest/formatting.html>`_ for the rules applied to
each table.

Project settings
----------------

Put per-file settings in ``[tool.pyproject-fmt]``:

.. code-block:: toml

    [tool.pyproject-fmt]
    column_width = 120
    indent = 2
    keep_full_version = false
    generate_python_version_classifiers = true
    max_supported_python = "3.14"
    table_format = "short"
    sub_table_spacing = ""
    separate_root_table = "\n"
    expand_tables = []
    collapse_tables = []
    skip_wrap_for_keys = []

These values match the command defaults. ``keep_full_version`` retains redundant zero components in dependency
versions. Classifier generation reads the lower bound from ``project.requires-python`` and uses
``max_supported_python`` as its upper bound.

``column_width`` controls array expansion and string wrapping. A trailing comma keeps an array multiline regardless of
its width. ``indent`` controls continuation indentation.

Shared settings
---------------

A standalone ``pyproject-fmt.toml`` can hold settings for several projects. The file uses the same keys without the
``[tool.pyproject-fmt]`` header:

.. code-block:: toml

    column_width = 120
    indent = 2
    table_format = "short"
    max_supported_python = "3.14"

For each input, the formatter searches from the input's directory toward the filesystem root and uses the nearest
``pyproject-fmt.toml``. ``--config`` selects a file directly:

.. code-block:: bash

    pyproject-fmt --config /path/to/pyproject-fmt.toml pyproject.toml

Command-line values establish defaults, the shared file overrides them, and ``[tool.pyproject-fmt]`` has final
precedence. The formatter validates file settings with the command-line converters. An unknown key or invalid value
stops formatting and reports its source.

Command-line interface
----------------------

Python classifiers
------------------

Classifier generation adds ``Programming Language :: Python :: 3.X`` entries for supported minor releases. The
``requires-python`` constraint supplies the lower edge and ``max_supported_python`` caps the result. The formatter
interprets the constraint under :pep:`440`:

- ``~=3.10`` includes 3.10 and later minor releases up to the configured cap.
- ``~=3.10.0`` includes the 3.10 series.
- ``!=3.10`` excludes that release, not the complete 3.10 series.
- A constraint with no matching Python 3 release, such as ``>=4``, produces no version classifiers.

Use ``generate_python_version_classifiers = false`` or ``--no-generate-python-version-classifiers`` to retain the
input classifier list.

Table shape
-----------

``table_format = "short"`` folds child tables into dotted keys. ``"long"`` writes child headers. Array-of-table
entries fold into inline tables when each entry fits within ``column_width``.

``expand_tables`` and ``collapse_tables`` override the default by table path. The closest matching path wins, with
``collapse_tables`` winning a tie. This configuration collapses most project children but expands optional
dependencies:

.. code-block:: toml

    [tool.pyproject-fmt]
    table_format = "long"
    collapse_tables = ["project"]
    expand_tables = ["project.optional-dependencies"]

The result folds ``project.urls`` and writes ``[project.optional-dependencies]`` as a header. Selectors also apply to
tool tables and arrays of tables.

Spacing
-------

``sub_table_spacing`` inserts text between child tables in one group. ``separate_root_table`` inserts text between root
groups. Each ``\n`` adds one blank line:

.. code-block:: toml

    [tool.pyproject-fmt]
    sub_table_spacing = "\n"
    separate_root_table = "\n"

String wrapping
---------------

``skip_wrap_for_keys`` excludes matching keys from line-continuation wrapping:

.. code-block:: toml

    [tool.pyproject-fmt]
    skip_wrap_for_keys = ["*.parse", "*.regex", "tool.bumpversion.*"]

Patterns match dotted key segments:

- ``tool.bumpversion.parse`` names one key.
- ``*.parse`` names any path ending in ``parse``.
- ``tool.bumpversion.*`` names direct children of ``tool.bumpversion``.
- ``tool.*.parse`` names one segment between ``tool`` and ``parse``.
- ``*`` names every key.

A quoted ``"*"`` segment names a literal asterisk.

This reference separates file-wide rules from the key order and array policy for recognized tables. See the
`configuration reference <https://pyproject-fmt.readthedocs.io/en/latest/configuration.html>`_ for available settings.

General Formatting
------------------

These rules cover the complete ``pyproject.toml`` file.

Table Ordering
~~~~~~~~~~~~~~

The formatter writes tables in this order:

1. ``[build-system]``
2. ``[project]``
3. ``[dependency-groups]``
4. ``[tool.*]`` sections in the order:

   1. Build backends: ``poetry``, ``poetry-dynamic-versioning``, ``pdm``, ``setuptools``, ``distutils``,
      ``setuptools_scm``, ``hatch``, ``flit``, ``scikit-build``, ``meson-python``, ``maturin``, ``pixi``,
      ``whey``, ``py-build-cmake``, ``sphinx-theme-builder``, ``uv``
   2. Builders: ``cibuildwheel``, ``nuitka``
   3. Linters/formatters: ``autopep8``, ``black``, ``yapf``, ``djlint``, ``ruff``, ``isort``, ``flake8``,
      ``pycln``, ``nbqa``, ``pylint``, ``repo-review``, ``codespell``, ``docformatter``, ``pydoclint``,
      ``interrogate``, ``tomlsort``, ``check-manifest``, ``check-sdist``, ``check-wheel-contents``, ``deptry``,
      ``vulture``, ``pyproject-fmt``, ``typos``, ``bandit``
   4. Type checkers: ``mypy``, ``pyrefly``, ``pyright``, ``ty``, ``django-stubs``
   5. Testing: ``pytest``, ``pytest_env``, ``pytest-enabler``, ``coverage``
   6. Task runners: ``doit``, ``spin``, ``tox``
   7. Release tools: ``bumpversion``, ``commitizen``, ``jupyter-releaser``, ``semantic_release``, ``tbump``,
      ``towncrier``, ``vendoring``
   8. Any other ``tool.*`` in alphabetical order

5. Other tables, alphabetically

String Quotes
~~~~~~~~~~~~~

Strings use double quotes unless the value contains one:

.. code-block:: toml

   # Before
   name = 'my-package'
   description = "He said \"hello\""

   # After
   name = "my-package"
   description = 'He said "hello"'

Key Quotes
~~~~~~~~~~

The formatter removes quotes from bare keys containing ``A-Za-z0-9_-``. Keys that need quotes use escaped double
quotes. The rule covers headers, assignments, and inline tables:

.. code-block:: toml

   # Before
   [tool."ruff"]
   "line-length" = 120
   lint.per-file-ignores.'tests/*' = ["S101"]

   # After
   [tool.ruff]
   line-length = 120
   lint.per-file-ignores."tests/*" = [ "S101" ]

Conversion escapes backslashes and double quotes in literal keys:

.. code-block:: toml

   # Before
   lint.per-file-ignores.'path\to\file' = ["E501"]

   # After
   lint.per-file-ignores."path\\to\\file" = [ "E501" ]

Array Formatting
~~~~~~~~~~~~~~~~

Short arrays stay on one line:

.. code-block:: toml

   # Before
   keywords = ["python", "toml"]

   # After
   keywords = [ "python", "toml" ]

An array that exceeds ``column_width`` expands and gains a trailing comma:

.. code-block:: toml

   # Before
   [project]
   keywords = ["web", "toml", "pyproject", "formatting"]

   # After
   [project]
   keywords = [
     "formatting",
     "pyproject",
     "toml",
     "web",
   ]

A trailing comma retains multiline form at any width:

.. code-block:: toml

   # Before
   classifiers = ["Development Status :: 4 - Beta",]

   # After
   classifiers = [
     "Development Status :: 4 - Beta",
   ]

A member comment also retains multiline form:

.. code-block:: toml

   lint.ignore = [
     "E501", # too long
     "E701",
   ]

An array uses multiline form when it has a trailing comma, exceeds ``column_width``, or contains a member comment.

String Wrapping
~~~~~~~~~~~~~~~

The formatter wraps a string that pushes its line past ``column_width``. Continuations account for the key or nested
indent. A key wider than the limit keeps its value on one line because wrapping cannot shorten that prefix.

.. code-block:: toml

   # Before
   description = "Format your pyproject.toml file in place"

   # After
   description = """\
     Format your pyproject.toml file in \
     place\
     """

Wrapping prefers spaces and ``" :: "`` separators. It skips inline-table strings and strings containing newlines.
``skip_wrap_for_keys`` excludes selected paths.

.. _table-formatting:

Table Formatting
~~~~~~~~~~~~~~~~

``table_format`` selects a child-table shape.

Short form uses dotted keys:

.. code-block:: toml

   [project]
   urls.homepage = "https://example.com"
   urls.repository = "https://github.com/example/project"

Long form uses headers:

.. code-block:: toml

   [project.urls]
   homepage = "https://example.com"
   repository = "https://github.com/example/project"

Child headers follow the same rank as their dotted keys. Unlisted children follow alphabetically, so
``[tool.coverage.run]`` precedes ``[tool.coverage.report]`` in long form just as ``run.*`` precedes ``report.*`` in
short form:

.. code-block:: toml

   # Before
   [tool.coverage.report]
   skip_covered = true

   [tool.coverage.run]
   branch = true

   # After
   [tool.coverage.run]
   branch = true
   [tool.coverage.report]
   skip_covered = true

Root groups have one blank line between them. Child tables stay adjacent unless ``sub_table_spacing`` adds a gap:

.. code-block:: toml

   # Before
   [tool.ruff]
   line-length = 120

   [tool.ruff.lint]
   select = ["E", "W"]

   # After
   [tool.ruff]
   line-length = 120

   [tool.ruff.lint]
   select = [ "E", "W" ]

See the `configuration reference <https://pyproject-fmt.readthedocs.io/en/latest/configuration.html>`_ for table
overrides and spacing.

.. _array-of-tables:

Array of Tables
~~~~~~~~~~~~~~~

Short form folds an array of tables when each entry fits within ``column_width``:

.. code-block:: toml

    # Before
    [[tool.commitizen.customize.questions]]
    type = "list"

    [[tool.commitizen.customize.questions]]
    type = "input"

    # After (with table_format = "short")
    [tool.commitizen]
    customize.questions = [ { type = "list" }, { type = "input" } ]

If one entry exceeds the limit, ``[[...]]`` headers remain because TOML 1.0 inline tables cannot span lines.

Comment Preservation
~~~~~~~~~~~~~~~~~~~~

Comments move with the value or entry they describe. Within an array, trailing comments align against that array's
longest value:

.. code-block:: toml

   # Before
   lint.ignore = [
     "COM812", # Conflict with formatter
     "CPY", # No copyright statements
     "ISC001",   # Another rule
   ]

   # After
   lint.ignore = [
     "COM812", # Conflict with formatter
     "CPY",    # No copyright statements
     "ISC001", # Another rule
   ]

Disabled Keys
~~~~~~~~~~~~~

A comment containing one valid assignment, such as ``# default = true``, represents a disabled field. The formatter
temporarily enables the assignment, formats it with its table, and restores the comment marker. This keeps the field
beside its active peers:

.. code-block:: toml

   # Before
   [[tool.uv.index]]
   name = "pypi"
   authenticate = "never"
   # default = true
   # ignore-error-codes = [400,401,403]

   # After
   [[tool.uv.index]]
   name = "pypi"
   authenticate = "never"
   # default = true
   # ignore-error-codes = [ 400, 401, 403 ]

Prose, multiline blocks, and commented headers remain ordinary comments. The check is structural: prose that parses as
one assignment receives disabled-field formatting. Rephrase such prose to avoid that interpretation. An
assignment wider than ``column_width`` also remains an ordinary comment.

Group Markers
~~~~~~~~~~~~~

An isolated ``# Group:`` comment divides an array, table, or section list into independent sort ranges. Group order and
the marker position stay fixed. Matching ignores case; trailing comments do not create boundaries.

The formatter sorts the entries inside each group:

.. code-block:: toml

   # Before
   [project]
   dependencies = [
     # Group: web
     "flask",
     "django",
     # Group: db
     "sqlalchemy",
     "psycopg2",
   ]

   # After
   [project]
   dependencies = [
     # Group: web
     "django",
     "flask",
     # Group: db
     "psycopg2",
     "sqlalchemy",
   ]

The same marker can precede a table key or ``[tool.*]`` header.

Line Endings
~~~~~~~~~~~~

Output retains the input's line ending. Mixed files use the more frequent ending, with ties resolved to ``\n``. Stdout
uses ``\n``.

Table-Specific Handling
-----------------------

Recognized tables add the rules below.

``[build-system]``
~~~~~~~~~~~~~~~~~~

The :pep:`517` / :pep:`518` table declares the project's build process. See the
`packaging specification <https://packaging.python.org/en/latest/specifications/pyproject-toml/#pyproject-build-system-table>`_.

Keys follow ``build-backend`` → ``requires`` → ``backend-path``. ``requires`` receives normalized spelling and
package-name order.


**Key ordering:** ``build-backend`` → ``requires`` → ``backend-path``

**Value normalization:**

- ``requires``: :pep:`508` spelling and package-name order
- ``backend-path``: input order, which controls the frontend's search

**Preserved as written:** every requirement the file declares. Setuptools has bundled ``bdist_wheel`` since
70.1, so a ``wheel`` entry beside it can be redundant, but no specifier says which release a resolver will
pick for a given build, and removing a dependency the author declared can leave that build unable to run.

.. code-block:: toml

   # Before
   [build-system]
   requires = ["setuptools >= 45", "wheel"]
   build-backend = "setuptools.build_meta"

   # After
   [build-system]
   build-backend = "setuptools.build_meta"
   requires = [ "setuptools>=45", "wheel" ]

``[project]``
~~~~~~~~~~~~~

The :pep:`621` core metadata table. See the
`packaging specification <https://packaging.python.org/en/latest/specifications/pyproject-toml/#pyproject-project-table>`_.

Keys follow the canonical metadata order. The formatter normalizes the name, dependency arrays, classifiers, and
keywords, and validates the version.


**Key ordering:** ``name`` → ``version`` → ``import-names`` → ``import-namespaces`` → ``description`` →
``readme`` → ``keywords`` → ``license`` → ``license-files`` → ``maintainers`` → ``authors`` →
``requires-python`` → ``classifiers`` → ``dynamic`` → ``dependencies`` → ``optional-dependencies`` →
``urls`` → ``scripts`` → ``gui-scripts`` → ``entry-points``

**Field normalizations:**

``name``
    Converted to canonical format (lowercase with hyphens): ``My_Package`` → ``my-package``

``version``
    Kept verbatim, because it is the exact version published in the package metadata, and normalizing would rewrite
    for example, CalVer ``2026.08.10`` to ``2026.8.10``. The formatter rejects values outside :pep:`440`, reports
    the error, and leaves the file untouched.

``description``
    Whitespace normalized: multiple spaces collapsed, consistent spacing after periods.

``license``
    Uppercases license expression operators (``and``, ``or``, ``with``): ``MIT or Apache-2.0`` →
    ``MIT OR Apache-2.0``. The formatter rewrites a value after it parses as an SPDX expression over registered
    license and exception identifiers, so free-form text that happens to read like one
    (``MIT or later``) retains its input spelling.

``requires-python``
    Whitespace removed: ``>= 3.9`` → ``>=3.9``

``keywords``
    Deduplicated (case-insensitive) and sorted alphabetically.

``dynamic``
    Sorted alphabetically.

``import-names`` / ``import-namespaces``
    Uses :pep:`794` spelling: a dotted name of Python identifiers followed by its optional modifier
    (``pkg.sub ;private`` → ``pkg.sub; private``). Valid entries sort alphabetically; other values retain their
    input spelling.

``classifiers``
    Deduplicated and sorted alphabetically.

``authors`` / ``maintainers``
    Retain published order. Each entry uses ``name`` → ``email`` key order.

**Dependency normalization:** dependency arrays use :pep:`508` spelling and canonical package-name order. The
formatter removes spaces and redundant ``.0`` suffixes unless ``keep_full_version = true``:

.. code-block:: toml

   # Before
   [project]
   dependencies = ["requests >= 2.0.0", "click~=8.0"]

   # After
   [project]
   dependencies = [ "click~=8.0", "requests>=2" ]

A direct-reference dependency keeps a space before its marker separator, because :pep:`508` only ends the URL
at whitespace; without it, installers read the ``;`` and the marker as part of the URL and reject the entry:

.. code-block:: toml

   # Before
   [project]
   dependencies = ["pkg @ git+https://github.com/user/repo.git@main ; python_version>='3.10'"]

   # After
   [project]
   dependencies = [ "pkg @ git+https://github.com/user/repo.git@main ; python_version>='3.10'" ]

**Optional-dependency extra names** use lowercase with hyphens:

.. code-block:: toml

   # Before
   [project.optional-dependencies]
   Dev_Tools = ["pytest"]

   # After
   [project]
   optional-dependencies.dev-tools = [ "pytest" ]

**Python version classifiers** derive from ``requires-python`` and ``max_supported_python`` (here ``3.14``).
Disable generation with ``generate_python_version_classifiers = false``:

.. code-block:: toml

   # Before
   [project]
   requires-python = ">=3.10"

   # After
   [project]
   requires-python = ">=3.10"
   classifiers = [
     "Programming Language :: Python :: 3 :: Only",
     "Programming Language :: Python :: 3.10",
     "Programming Language :: Python :: 3.11",
     "Programming Language :: Python :: 3.12",
     "Programming Language :: Python :: 3.13",
     "Programming Language :: Python :: 3.14",
   ]

**Entry points:** inline tables within ``entry-points`` expand to dotted keys:

.. code-block:: toml

   # Before
   [project]
   entry-points.console_scripts = { mycli = "mypackage:main" }

   # After
   [project]
   entry-points.console_scripts.mycli = "mypackage:main"

**Authors / maintainers** can be inline tables (short format):

.. code-block:: toml

   # Before
   [project]
   authors = [{ name = "Alice", email = "alice@example.com" }]

   # After
   [project]
   authors = [ { name = "Alice", email = "alice@example.com" } ]

or an expanded array of tables (long format, controlled by ``table_format``, ``expand_tables``, and
``collapse_tables``):

.. code-block:: toml

   [[project.authors]]
   name = "Alice"
   email = "alice@example.com"

``[dependency-groups]``
~~~~~~~~~~~~~~~~~~~~~~~

The :pep:`735` table for named groups of development dependencies. See the
`packaging specification <https://packaging.python.org/en/latest/specifications/dependency-groups/>`_.

Groups follow ``dev`` → ``test`` → ``type`` → ``docs`` → other names alphabetically. Each group receives normalized
dependency spelling and package-name order.


**Key ordering:** ``dev`` → ``test`` → ``type`` → ``docs`` → others alphabetically

**Value normalization:**

- all dependencies normalized per :pep:`508`
- an ``include-group`` pulls its group into its current position; requirements between two inclusions sort

.. code-block:: toml

   # Before
   [dependency-groups]
   dev = [{ include-group = "test" }, "ruff>=0.4", "mypy>=1"]

   # After
   [dependency-groups]
   dev = [ { include-group = "test" }, "mypy>=1", "ruff>=0.4" ]

``[tool.poetry]``
~~~~~~~~~~~~~~~~~

`Poetry <https://python-poetry.org/>`_ is a Python dependency management and packaging tool. See its
`pyproject.toml reference <https://python-poetry.org/docs/pyproject/>`_.

Covers Poetry 1.x metadata under ``[tool.poetry]`` and Poetry 2.x tool-specific keys. Sections and inline tables follow
Poetry's documented order. Set-like arrays sort; sequence-dependent arrays retain input order.


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
    ``default`` and ``secondary`` last. Entries retain priority order.

**Sorted arrays:**

- ``keywords``, ``classifiers``: deduplicated (case-insensitive) and sorted alphabetically.
- ``exclude``: sorted alphabetically.
- ``[tool.poetry.extras]`` values (each ``extras.<name>``): sorted alphabetically.
- ``[tool.poetry.group.<name>.include-groups]``: sorted alphabetically.
- Per-dependency ``extras`` arrays (in ``dependencies``, ``dev-dependencies``, per-group dependencies,
  ``requires-plugins``, ``build-constraints``): sorted alphabetically.

**Preserved order:** ``authors``, ``maintainers``, ``packages``,
``include``, ``readme`` (when an array), multi-constraint dependency arrays, and ``[[tool.poetry.source]]``
entries.

**Inline-table key ordering:** Poetry discriminator keys select one of these orders:

- Sources (``{ priority = ... }``, ``{ secondary = ... }``, ``{ links = ... }``, ``{ indexed = ... }``):
  ``name`` → ``url`` → ``priority`` → ``links`` → ``indexed`` → ``default`` → ``secondary``.
- Git dependencies (``{ git = ... }``):
  ``git`` → ``branch`` → ``tag`` → ``rev`` → ``subdirectory`` → ``python`` → ``platform`` → ``markers`` →
  ``allow-prereleases`` → ``allows-prereleases`` → ``optional`` → ``extras`` → ``develop``.
- Path dependencies (``{ path = ... }``):
  ``path`` → ``develop`` → ``subdirectory`` → ``python`` → ``platform`` → ``markers`` → ``optional`` →
  ``extras``.
- File dependencies (``{ file = ... }``):
  ``file`` → ``subdirectory`` → ``python`` → ``platform`` → ``markers`` → ``optional`` → ``extras``.

Inline tables outside these schemas, such as ``{ name = "...", email = "..." }``, retain their input order.

.. code-block:: toml

   # Before
   [[tool.poetry.source]]
   priority = "primary"
   url = "https://example.com"
   name = "private"

   [tool.poetry.dependencies]
   zebra = "^1.0"
   python = "^3.11"
   foo = { branch = "main", git = "https://example.com/foo" }

   # After
   [tool.poetry]
   dependencies.python = "^3.11"
   dependencies.foo = { git = "https://example.com/foo", branch = "main" }
   dependencies.zebra = "^1.0"
   source = [ { name = "private", url = "https://example.com", priority = "primary" } ]

``[tool.pdm.*]``
~~~~~~~~~~~~~~~~

`PDM <https://pdm-project.org/latest/>`_ is a modern Python package and dependency manager. See its
`build configuration reference <https://pdm-project.org/latest/reference/build/>`_.

Top-level keys follow distribution → resolution → version → build → scripts → source → dev-dependencies → publish →
options. Name and glob arrays sort; source entries retain priority order.


**Top-level key ordering:** distribution / package-type / plugins → resolution → version → build → scripts →
source → dev-dependencies → publish → options.

**Sub-table ordering** (collapsed to dotted keys):

- ``version``: ``source`` → ``path`` → ``getter`` → ``write_to`` → ``write_template`` → ``tag_regex`` →
  ``tag_filter`` → ``fallback_version`` → ``version_format``.
- ``build``: ``includes`` → ``excludes`` → ``source-includes`` → ``package-dir`` → ``is-purelib`` →
  ``run-setuptools`` → ``custom-hook`` → ``editable-backend``.
- ``[[tool.pdm.source]]`` (array of tables, order preserved): per-entry ``name`` → ``url`` → ``type`` →
  ``verify_ssl`` → ``include_packages`` → ``exclude_packages``.

**Sorted arrays:** ``plugins``, ``build.includes``, ``build.excludes``, ``build.source-includes``,
``resolution.excludes``, every ``dev-dependencies.<group>`` value array, and ``include_packages`` /
``exclude_packages`` inside source entries.

``[tool.setuptools]`` and ``[tool.setuptools_scm]``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

`setuptools <https://setuptools.pypa.io/en/latest/>`_ is a build backend and packaging library;
`setuptools_scm <https://setuptools-scm.readthedocs.io/en/latest/>`_ derives the package version from SCM tags. See
the setuptools `pyproject.toml reference <https://setuptools.pypa.io/en/latest/userguide/pyproject_config.html>`_
and the setuptools_scm `configuration reference <https://setuptools-scm.readthedocs.io/en/latest/config/>`_.

Both tables group keys by discovery → data → metadata → deprecated. Name and glob arrays sort; literal lists such as
``packages`` retain input order.


``[tool.setuptools]`` top-level key ordering (grouped):

1. Packaging discovery: ``py-modules`` → ``packages.find.*`` / ``packages.find-namespace.*`` → ``packages`` →
   ``package-dir``
2. Package data: ``include-package-data`` → ``package-data`` → ``exclude-package-data``
3. Dynamic metadata: ``dynamic``
4. Extensions / build customization: ``ext-modules`` → ``cmdclass``
5. Distribution metadata: ``platforms`` → ``provides`` → ``obsoletes`` → ``license-files``
6. Data files: ``data-files``
7. Deprecated / obsolete (pushed last): ``script-files`` → ``namespace-packages`` → ``zip-safe`` →
   ``eager-resources`` → ``dependency-links``

``[tool.setuptools.packages.find]`` / ``[tool.setuptools.packages.find-namespace]`` inner ordering: ``where`` →
``include`` → ``exclude`` → ``namespaces``.

``[tool.setuptools.package-data]`` / ``[tool.setuptools.exclude-package-data]`` / ``[tool.setuptools.data-files]``
ordering: the catch-all ``"*"`` pattern always goes first, then the other package patterns alphabetically; each
value sorts alphabetically.

``[tool.setuptools.dynamic]`` ordering: field names alphabetized. Inline-table directives (e.g.
``version = { attr = "pkg.__version__" }`` or ``readme = { file = "README.md", content-type = "text/markdown" }``)
get their keys ordered ``attr`` → ``file`` → ``content-type``.

**Sorted arrays:**

- ``py-modules``, ``platforms``, ``provides``, ``obsoletes``, ``namespace-packages``, ``eager-resources``:
  alphabetized.
- ``packages.find.include`` / ``packages.find.exclude`` / ``packages.find-namespace.*``: alphabetized.
- Values inside ``package-data`` / ``exclude-package-data`` tables: alphabetized.

**Preserved order:** ``packages`` (literal list, first match wins),
``license-files`` (PEP 639 concatenation order), ``script-files`` and the ``data-files`` lists (installed in
order, which decides which of two files sharing a name reaches the installation), and everything under
``[[tool.setuptools.ext-modules]]`` (compiler and linker argv arrays).

``[tool.setuptools_scm]`` key ordering (grouped):

1. Version output: ``version_file`` → ``version_file_template``
2. Version computation: ``version_scheme`` → ``local_scheme`` → ``version_cls`` → ``normalize``
3. Root discovery: ``root`` → ``relative_to`` → ``fallback_root`` → ``parent`` →
   ``search_parent_directories`` → ``dist_name``
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
   dynamic.readme = { file = "README.md", content-type = "text/markdown" }
   zip-safe = false

``[tool.hatch.*]``
~~~~~~~~~~~~~~~~~~

`Hatch <https://hatch.pypa.io/latest/>`_ is a Python project manager based on the Hatchling build backend. See its
`build configuration reference <https://hatch.pypa.io/latest/config/build/>`_.

Hatch tables group keys by version → metadata → build → publish → workspace → environments. Name and path arrays sort;
build hooks and matrix entries retain input order.


**Key ordering:** keys at ``[tool.hatch]`` level (after collapse, dotted ``version.*`` / ``build.*`` /
``metadata.*`` / ``envs.*`` / ``publish.*`` / ``workspace.*``):

1. Version: ``version.source`` → ``version.path`` → ``version.pattern`` → ``version.expression`` →
   ``version.scheme`` → ``version.validate-bump`` → ``version.fallback-version`` → ``version.raw-options``.
2. Metadata: ``metadata.allow-direct-references`` → ``metadata.allow-ambiguous-features`` → ``metadata.hooks``.
3. Build: ``build.dev-mode-dirs`` → ``build.directory`` → ``build.sources`` → ``build.packages`` →
   ``build.include`` → ``build.exclude`` → ``build.force-include`` → ``build.artifacts`` →
   ``build.ignore-vcs`` → ``build.skip-excluded-dirs`` → ``build.reproducible`` → ``build.hooks`` → wheel
   target (``packages``, ``include``, ``exclude``, ``force-include``, ``artifacts``, ``hooks``, ``shared-data``,
   ``extra-metadata``, etc.) → sdist target (``include``, ``exclude``, ``force-include``, ``support-legacy``,
   ``strict-naming``).
4. Publish: ``publish.index.disable`` → ``publish.index.repos`` → ``publish.index``.
5. Workspace: ``workspace.members`` → ``workspace.exclude``.
6. Environments (``envs.<name>.*``): each environment's keys follow ``type`` → ``template`` → ``detached`` →
   ``description`` → ``platforms`` → ``python`` → ``path`` → ``installer`` → ``skip-install`` →
   ``system-packages`` → ``dev-mode`` → ``features`` → ``dependencies`` → ``extra-dependencies`` →
   ``extra-args`` → ``pre-install-commands`` → ``post-install-commands`` → ``env-include`` → ``env-exclude``
   → ``env-vars`` → ``scripts`` → ``matrix`` → ``matrix-name-format`` → ``overrides``.

**Sorted arrays:**

- Build: ``packages``, ``sources``, ``dev-mode-dirs``, and ``build.targets.wheel.packages``. ``include``,
  ``exclude``, ``force-include`` and ``artifacts`` keep their order, since hatch reads them the way a gitignore is
  read, where a ``!pattern`` after a broader one takes back what it matched.
- Environments: per-env ``dependencies``, ``extra-dependencies``, ``features``, ``platforms``,
  ``env-include``, ``env-exclude``. ``pre-install-commands`` and ``post-install-commands`` keep their order, since
  hatch runs them in list order.
- Workspace: ``members``, ``exclude``.

``scripts`` and ``env-vars`` sub-tables under each environment have their inner keys alphabetized.

**Preserved as written:** build-hook order and matrix entry order (both carry semantic meaning).

``[tool.scikit-build]``
~~~~~~~~~~~~~~~~~~~~~~~

`scikit-build-core <https://scikit-build-core.readthedocs.io/en/latest/>`_ is a CMake-based build backend for
Python C/C++ extensions. See its `configuration reference
<https://scikit-build-core.readthedocs.io/en/latest/configuration/index.html>`_.

Keys follow meta → build → cmake → ninja → sdist → wheel → install → editable → logging → metadata → search →
``generate`` → ``overrides``. Name and path lists sort; cmake and ninja arguments retain input order.


**Key ordering:** meta keys (``minimum-version``, ``build-dir``, ``fail``, ``experimental``,
``strict-config``) → ``build`` → ``cmake`` → ``ninja`` → ``sdist`` → ``wheel`` → ``install`` → ``editable`` →
``logging`` / ``messages`` → ``metadata`` → ``search`` → ``generate`` (array of tables) → ``overrides``
(array of tables).

**Sorted arrays:** ``files``, ``exclude-fields``.

**Preserved as written:** ``packages`` (a later path can replace what an earlier one installed), ``include`` and
``exclude`` (read the way a gitignore is read, where a later negation takes back an earlier match), ``targets`` and
``components`` (cmake runs and installs them in order), and ``args`` and ``define`` (CLI argv for cmake/ninja).

``[tool.maturin]``
~~~~~~~~~~~~~~~~~~

`Maturin <https://www.maturin.rs/>`_ builds and publishes Rust-based Python extension modules. See its
`configuration reference <https://www.maturin.rs/config>`_.

Keys follow module identity → source layout → cargo settings → compatibility/strip → behavior. Set-like arrays sort;
cargo and rustc arguments retain input order.


**Key ordering:** module identity (``module-name``, ``bindings``, ``python-source``, ``python-packages``,
``python-bin-path``) → source layout (``src``, ``manifest-path``, ``include``, ``exclude``, ``sdist-generator``,
``data``) → cargo settings (``features``, ``no-default-features``, ``all-features``, ``rustc-args``,
``unstable-flags``, ``config``, ``profile``, ``target``, ``target-dir``) → compatibility / strip
(``compatibility``, ``auditwheel``, ``skip-auditwheel``, ``strip``, ``include-import-lib``, ``frozen``,
``locked``, ``offline``, ``zig``) → behavior (``use-cross``, ``use-base-python``).

**Sorted arrays:** ``python-packages``, ``include``, ``features`` (all set semantics).

**Preserved as written:** ``exclude`` (an ordered override program, where a ``!pattern`` after a broader one takes
back what it matched) and ``rustc-args`` / ``unstable-flags`` (CLI argv).

``[tool.pixi]``
~~~~~~~~~~~~~~~

`Pixi <https://pixi.prefix.dev/latest/>`_ is a cross-platform conda/PyPI package and environment manager. See its
`pyproject.toml reference <https://pixi.prefix.dev/latest/python/pyproject_toml/>`_.

Keys follow workspace metadata → configuration → dependencies → environments → build. A platform array containing
plain names sorts.


**Key ordering:**

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

**Sorted arrays:** ``workspace.platforms`` and ``workspace.preview``, where every entry is a plain name.

**Preserved as written:** ``workspace.channels`` and ``workspace.build-variants-files``, since pixi reads both in
input order and lets the earlier entry win, and a ``workspace.platforms`` holding a rich platform
table, since that names no platform to sort by and pixi runs the first entry the host satisfies.

``[tool.uv]``
~~~~~~~~~~~~~

`uv <https://docs.astral.sh/uv/>`_ is Astral's Python package and project manager. See its
`settings reference <https://docs.astral.sh/uv/reference/settings/>`_.

Keys follow Python → dependencies → sources → resolution → build → network → publishing → workspace. Package-name
arrays and the ``sources`` table sort alphabetically.


**Key ordering:**

1. Version & Python: ``required-version`` → ``python-preference`` → ``python-downloads``
2. Dependencies: ``dev-dependencies`` → ``default-groups`` → ``dependency-groups`` →
   ``constraint-dependencies`` → ``override-dependencies`` → ``exclude-dependencies`` → ``dependency-metadata``
3. Sources & indexes: ``sources`` → ``index`` → ``index-url`` → ``extra-index-url`` → ``find-links`` →
   ``no-index`` → ``index-strategy`` → ``keyring-provider``
4. Package handling: ``no-binary*`` → ``no-build*`` → ``no-sources*`` → ``reinstall*`` → ``upgrade*``
5. Resolution: ``resolution`` → ``prerelease`` → ``fork-strategy`` → ``environments`` →
   ``required-environments`` → ``exclude-newer*``
6. Build & Install: ``compile-bytecode`` → ``link-mode`` → ``config-settings*`` → ``extra-build-*`` →
   ``concurrent-builds`` → ``concurrent-downloads`` → ``concurrent-installs``
7. Network & Security: ``allow-insecure-host`` → ``native-tls`` → ``offline`` → ``no-cache`` → ``cache-dir`` →
   ``http-proxy`` → ``https-proxy`` → ``no-proxy``
8. Publishing: ``publish-url`` → ``check-url`` → ``trusted-publishing``
9. Python management: ``python-install-mirror`` → ``pypy-install-mirror`` → ``python-downloads-json-url``
10. Workspace & Project: ``managed`` → ``package`` → ``workspace`` → ``conflicts`` → ``cache-keys`` →
    ``build-backend``
11. Other: ``pip`` → ``preview`` → ``torch-backend``

**Sorted arrays:**

Package-name arrays
  ``constraint-dependencies``, ``override-dependencies``, ``dev-dependencies``, ``exclude-dependencies``,
  ``no-binary-package``, ``no-build-package``, ``no-build-isolation-package``, ``no-sources-package``,
  ``reinstall-package``, ``upgrade-package``

Other arrays
  ``environments``, ``required-environments``, ``allow-insecure-host``, ``no-proxy``, ``workspace.members``,
  ``workspace.exclude``

**Sources table:** entries sort by package name:

.. code-block:: toml

   # Before
   [tool.uv.sources]
   zebra = { git = "..." }
   alpha = { path = "..." }

   # After
   [tool.uv]
   sources.alpha = { path = "..." }
   sources.zebra = { git = "..." }

**pip subsection:** ``[tool.uv.pip]`` follows the same rules, with arrays like ``extra``, ``no-binary-package``,
``no-build-package``, ``reinstall-package``, and ``upgrade-package`` sorted alphabetically.

``[tool.cibuildwheel]``
~~~~~~~~~~~~~~~~~~~~~~~

`cibuildwheel <https://cibuildwheel.pypa.io/en/stable/>`_ builds Python wheels across platforms in CI. See its
`options reference <https://cibuildwheel.pypa.io/en/stable/options/>`_.

Keys follow selection → build config → build phases → test phases → platform images → per-platform sub-tables →
``overrides``. Set-like arrays sort; argument lists retain input order.


**Key ordering:** selection (``build``, ``skip``, ``test-skip``, ``archs``, ``enable``,
``free-threaded-support``) → build configuration (``build-frontend``, ``build-verbosity``, ``config-settings``,
``dependency-versions``, ``environment``, ``environment-pass``) → build phases (``before-all``,
``before-build``, ``repair-wheel-command``) → test phases (``before-test``, ``test-command``,
``test-requires``, ``test-extras``, ``test-groups``, ``test-sources``) → platform images
(``manylinux-*-image``, ``musllinux-*-image``) → ``container-engine`` → per-platform sub-tables (``linux``,
``macos``, ``windows``, ``android``, ``ios``, ``pyodide``) → ``overrides`` last.

Per-platform sub-tables follow the same inner ordering. ``overrides`` entries, whether written as
``[[tool.cibuildwheel.overrides]]`` or as inline tables in ``overrides = [...]``, place ``select`` first
(required), then the regular cibuildwheel keys. Entries retain order because later overrides win.

**Sorted arrays:** ``enable``, ``test-extras``, ``test-groups``.

**Preserved as written:** most other array-valued keys (``test-requires``, ``before-all``, ``test-command``,
the various ``environment*`` fields) are CLI argv or ordered lists.

``[tool.autopep8]``
~~~~~~~~~~~~~~~~~~~

`autopep8 <https://github.com/hhatto/autopep8>`_ formats Python code to conform to PEP 8. See its
`configuration reference <https://github.com/hhatto/autopep8#pyproject-toml>`_.

Keys follow length/indent → mode → rules → behavior. Rule lists sort.


**Key ordering:** length/indent → mode (``in-place``, ``recursive``, ``diff``, ``list-fixes``) → rules
(``ignore``, ``select``, ``exclude``) → behavior.

**Sorted arrays:** ``ignore``, ``select``, ``exclude``.

``[tool.black]``
~~~~~~~~~~~~~~~~

`Black <https://black.readthedocs.io/en/stable/>`_ is a Python code formatter. See its
`configuration reference <https://black.readthedocs.io/en/stable/usage_and_configuration/the_basics.html>`_.

Keys follow Black's option groups. ``target-version`` and ``enable-unstable-feature`` sort alphabetically.


**Key ordering:**

1. ``required-version`` → ``target-version`` → ``line-length``
2. File selection: ``include`` → ``extend-exclude`` → ``force-exclude`` → ``exclude``
3. Behavior: ``skip-string-normalization`` → ``skip-magic-trailing-comma`` → ``preview`` → ``unstable`` →
   ``enable-unstable-feature`` → ``fast`` → ``workers``
4. Output: ``color`` → ``verbose`` → ``quiet``

**Sorted arrays:** ``target-version`` (so ``py39`` precedes ``py310``), ``enable-unstable-feature``.

The ``include`` and ``exclude`` family hold regex strings, so they retain input spelling.

``[tool.yapf]``
~~~~~~~~~~~~~~~

`YAPF <https://github.com/google/yapf>`_ is a configurable Python code formatter from Google. See its
`configuration reference <https://github.com/google/yapf#knobs>`_.

A single flat table: ``based_on_style`` comes first (it sets the defaults), then the rest in a fixed order.


**Key ordering:** ``based_on_style`` first (sets defaults), then ``column_limit``, ``indent_width``,
``continuation_indent_width``, then the remaining keys alphabetized.

``[tool.djlint]``
~~~~~~~~~~~~~~~~~

`djLint <https://djlint.com/>`_ is a linter and formatter for HTML templates (Django, Jinja, and more). See its
`configuration reference <https://djlint.com/docs/configuration/>`_.

Keys follow profile/scope → formatting → linting → ignores → output. Exclude and block lists sort.


**Key ordering:** profile/scope → formatting → linting → ignores → output.

**Sorted arrays:** ``exclude``, ``extend_exclude``, ``custom_blocks``, ``custom_html``, ``ignore``,
``ignore_blocks``.

``[tool.ruff]``
~~~~~~~~~~~~~~~

`Ruff <https://docs.astral.sh/ruff/>`_ is a Python linter and formatter written in Rust. See its
`settings reference <https://docs.astral.sh/ruff/settings/>`_.

Keys follow Ruff's option grouping (global → paths → behavior → output → ``format`` → ``lint``); rule-code, path, and
name arrays use natural order (``RUF1`` < ``RUF9`` < ``RUF10``).


**Key ordering:**

1. Global settings: ``required-version`` → ``extend`` → ``target-version`` → ``line-length`` →
   ``indent-width`` → ``tab-size``
2. Path settings: ``builtins`` → ``namespace-packages`` → ``src`` → ``include`` → ``extend-include`` →
   ``exclude`` → ``extend-exclude`` → ``force-exclude`` → ``respect-gitignore``
3. Behavior flags: ``preview`` → ``fix`` → ``unsafe-fixes`` → ``fix-only`` → ``show-fixes`` → ``show-source``
4. Output settings: ``output-format`` → ``cache-dir``
5. ``format.*`` keys
6. ``lint.*`` keys: ``select`` → ``extend-select`` → ``ignore`` → ``extend-ignore`` → ``per-file-ignores`` →
   ``fixable`` → ``unfixable`` → plugin configurations

**Sorted arrays:** alphabetical with natural ordering (``RUF1`` < ``RUF9`` < ``RUF10``); per-file-ignores values
follow the same order:

.. code-block:: toml

   # Before
   [tool.ruff]
   lint.select = ["F", "E", "RUF", "I"]
   lint.ignore = ["E701", "E501"]
   lint.per-file-ignores."tests/*.py" = ["S101", "D103"]

   # After
   [tool.ruff]
   lint.select = [ "E", "F", "I", "RUF" ]
   lint.ignore = [ "E501", "E701" ]
   lint.per-file-ignores."tests/*.py" = [ "D103", "S101" ]

The full set of sorted array keys:

Top-level
  ``exclude``, ``extend-exclude``, ``include``, ``extend-include``, ``builtins``, ``namespace-packages``,
  ``src``

Format
  ``format.exclude``

Lint
  ``select``, ``extend-select``, ``ignore``, ``extend-ignore``, ``fixable``, ``extend-fixable``, ``unfixable``,
  ``extend-safe-fixes``, ``extend-unsafe-fixes``, ``external``, ``task-tags``, ``exclude``, ``typing-modules``,
  ``allowed-confusables``, ``logger-objects``

Per-file patterns
  ``lint.per-file-ignores.*``, ``lint.extend-per-file-ignores.*``

Plugin arrays
  ``lint.flake8-bandit.hardcoded-tmp-directory``, ``lint.flake8-bandit.hardcoded-tmp-directory-extend``,
  ``lint.flake8-boolean-trap.extend-allowed-calls``, ``lint.flake8-bugbear.extend-immutable-calls``,
  ``lint.flake8-builtins.builtins-ignorelist``, ``lint.flake8-gettext.extend-function-names``,
  ``lint.flake8-gettext.function-names``, ``lint.flake8-import-conventions.banned-from``,
  ``lint.flake8-pytest-style.raises-extend-require-match-for``,
  ``lint.flake8-pytest-style.raises-require-match-for``, ``lint.flake8-self.extend-ignore-names``,
  ``lint.flake8-self.ignore-names``, ``lint.flake8-tidy-imports.banned-module-level-imports``,
  ``lint.flake8-type-checking.exempt-modules``, ``lint.flake8-type-checking.runtime-evaluated-base-classes``,
  ``lint.flake8-type-checking.runtime-evaluated-decorators``, ``lint.isort.constants``,
  ``lint.isort.default-section``, ``lint.isort.extra-standard-library``,
  ``lint.isort.no-lines-before``, ``lint.isort.required-imports``, ``lint.isort.single-line-exclusions``,
  ``lint.isort.variables``, ``lint.pep8-naming.classmethod-decorators``,
  ``lint.pep8-naming.extend-ignore-names``, ``lint.pep8-naming.ignore-names``,
  ``lint.pep8-naming.staticmethod-decorators``, ``lint.pydocstyle.ignore-decorators``,
  ``lint.pydocstyle.property-decorators``, ``lint.pyflakes.extend-generics``,
  ``lint.pylint.allow-dunder-method-names``, ``lint.pylint.allow-magic-value-types``

**Preserved order:** ``lint.isort.forced-separate``, whose list order controls auxiliary import blocks.

``[tool.isort]``
~~~~~~~~~~~~~~~~

`isort <https://pycqa.github.io/isort/>`_ sorts and organizes Python imports. See its
`configuration options <https://pycqa.github.io/isort/docs/configuration/options.html>`_.

``profile`` comes first (it sets the defaults everything else overrides), then output style, known sources,
separation, skip patterns, and import edits. Name lists sort; sequence-dependent lists retain input order.


**Key ordering:**

1. ``profile``: sets defaults that the keys below override
2. Output style: line, wrap, indent, and multi-line options
3. Known sources: ``sections`` → ``default_section`` → ``known_standard_library`` →
   ``extra_standard_library`` → ``known_third_party`` → ``known_first_party`` → ``known_local_folder`` →
   ``known_other``
4. Forced separation, skip patterns, import add/remove, and section heading comments

**Sorted arrays:** ``known_standard_library``, ``extra_standard_library``, ``known_third_party``,
``known_first_party``, ``known_local_folder``, ``known_other``, ``namespace_packages``, ``src_paths``,
``skip``, ``skip_glob``, ``extend_skip``, ``extend_skip_glob``, ``supported_extensions``,
``blocked_extensions``, ``single_line_exclusions``, ``treat_comments_as_code``,
``treat_all_comments_as_code``, ``constants``, ``variables``.

**Preserved order:** ``sections`` (output section order), ``no_lines_before``,
``add_imports``, ``remove_imports``, ``required_imports``, ``force_to_top``, ``forced_separate`` (list order sets
group placement).

``[tool.pylint.*]``
~~~~~~~~~~~~~~~~~~~

`Pylint <https://pylint.readthedocs.io/en/stable/>`_ is a static analyzer and linter for Python. See
its `configuration reference <https://pylint.readthedocs.io/en/stable/user_guide/configuration/index.html>`_.

Sub-tables follow Pylint's checker-group order. Rule, name, and path lists sort by leaf key, independent of sub-table.


**Sub-table order:** ``main`` (and legacy alias ``master``) → ``messages_control`` → ``reports`` → ``basic``
→ ``format`` → ``design`` → ``classes`` → ``exceptions`` → ``imports`` → ``logging`` → ``method_args`` →
``refactoring`` → ``similarities`` → ``spelling`` → ``string`` → ``typecheck`` → ``variables`` →
``miscellaneous``.

**Sorted arrays:** ``enable``, ``disable``, ``load-plugins``, ``extension-pkg-allow-list``,
``extension-pkg-whitelist``, ``ignore``, ``ignore-patterns``, ``ignore-paths``, ``ignored-modules``,
``ignored-classes``, ``ignored-argument-names``, ``good-names``, ``bad-names``, ``logging-modules``,
``valid-classmethod-first-arg``, ``valid-metaclass-classmethod-first-arg``, ``callbacks``,
``additional-builtins``, ``allowed-redefined-builtins``, ``preferred-modules``, ``deprecated-modules``,
``known-third-party``, ``known-standard-library``, ``allowed-modules``, ``expected-line-ending-format``,
``overgeneral-exceptions``, ``defining-attr-methods``, ``exclude-protected``. Matching is on the leaf key name
regardless of which sub-table it appears in.

``[tool.codespell]``
~~~~~~~~~~~~~~~~~~~~

`codespell <https://github.com/codespell-project/codespell>`_ checks code and text for common misspellings. See its
`configuration reference <https://github.com/codespell-project/codespell#using-a-config-file>`_.

Keys follow dictionaries → scope → fix behavior → output. Word and path lists sort.


**Key ordering:** dictionaries (``builtin``, ``dictionary``, ``ignore-words``, ``ignore-words-list``,
``ignore-regex``, ``ignore-multiline-regex``, ``exclude-file``) → scope (``skip``, ``uri-ignore-words-list``,
``check-filenames``, ``check-hidden``, ``hidden``, ``regex``, ``user-input``) → fix behavior
(``write-changes``, ``interactive``, ``enable-colors``, ``disable-colors``) → output (``count``,
``quiet-level``, ``summary``).

**Sorted arrays:** ``builtin``, ``dictionary``, ``skip``, ``ignore-words-list``, ``uri-ignore-words-list``.

``[tool.docformatter]``
~~~~~~~~~~~~~~~~~~~~~~~

`docformatter <https://docformatter.readthedocs.io/en/latest/>`_ formats Python docstrings to follow PEP 257. See
its `configuration reference <https://docformatter.readthedocs.io/en/latest/configuration.html>`_.

Keys follow behavior → format width → wrap/summary tweaks → other.


**Key ordering:** behavior (``in-place``, ``recursive``, ``check``, ``diff``, ``black``, ``pep257``,
``non-strict``) → format width (``line-length``, ``wrap-summaries``, ``wrap-descriptions``, ``tab-width``) →
wrap/summary tweaks → other.

``[tool.interrogate]``
~~~~~~~~~~~~~~~~~~~~~~

`interrogate <https://interrogate.readthedocs.io/en/latest/>`_ measures docstring coverage of a Python codebase.
See its `configuration reference <https://interrogate.readthedocs.io/en/latest/#configuration>`_.

Keys follow threshold → ignore flags → exclude → output. Exclude and regex lists sort.


**Key ordering:** threshold → ignore flags → exclude → output.

**Sorted arrays:** ``exclude``, ``extend-exclude``, ``ignore-regex``.

``[tool.check-manifest]``
~~~~~~~~~~~~~~~~~~~~~~~~~

`check-manifest <https://github.com/mgedmin/check-manifest>`_ checks that ``MANIFEST.in`` is complete for an
sdist. See its `configuration reference <https://github.com/mgedmin/check-manifest#configuration>`_.

Keys follow ``ignore`` → ``ignore-bad-ideas`` → ``ignore-default-rules``. Both glob lists sort.


**Key ordering:** ``ignore`` → ``ignore-bad-ideas`` → ``ignore-default-rules``.

**Sorted arrays:** ``ignore`` and ``ignore-bad-ideas`` (file-glob lists).

``[tool.deptry]``
~~~~~~~~~~~~~~~~~

`deptry <https://deptry.com/>`_ finds unused, missing, and transitive dependencies in Python projects. See its
`usage reference <https://deptry.com/usage/>`_.

Keys follow scope/exclude → ignore rules → per-rule ignores → behavior → mapping. Ignore and path lists sort.


**Key ordering:** scope/exclude → ignore rules → per-rule ignores → behavior → mapping.

**Sorted arrays:** the ``ignore_*`` / ``exclude`` / ``requirements_files`` / ``pep621_dev_dependency_groups``
/ ``known_first_party`` lists.

``[tool.vulture]``
~~~~~~~~~~~~~~~~~~

`Vulture <https://github.com/jendrikseipp/vulture>`_ finds unused (dead) Python code. See its
`configuration reference <https://github.com/jendrikseipp/vulture#configuration>`_.

Keys follow paths → ignore → behavior → output. Path and name lists sort.


**Key ordering:** paths → ignore (``exclude``, ``ignore_names``, ``ignore_decorators``) → behavior
(``make_whitelist``, ``min_confidence``, ``sort_by_size``) → output (``verbose``).

**Sorted arrays:** ``paths``, ``exclude``, ``ignore_names``, ``ignore_decorators``.

``[tool.bandit]``
~~~~~~~~~~~~~~~~~

`Bandit <https://bandit.readthedocs.io/en/latest/>`_ finds common security issues in Python code. See its
`configuration reference <https://bandit.readthedocs.io/en/latest/config.html>`_.

Keys follow ``exclude_dirs`` → ``targets`` → ``tests`` → ``skips`` → per-plugin sub-tables. Array values sort
alphabetically.


**Key ordering:** ``exclude_dirs`` → ``targets`` → ``tests`` → ``skips`` → per-plugin sub-tables
(``assert_used``, ``hardcoded_tmp_directory``, etc.).

**Sorted arrays:** all array values (rule IDs, directory paths, function-name lists, all set semantics).

``[tool.mypy]``
~~~~~~~~~~~~~~~

`mypy <https://mypy.readthedocs.io/en/stable/>`_ is a static type checker for Python. See its
`configuration reference <https://mypy.readthedocs.io/en/stable/config_file.html>`_.

Covers mypy's documented options and ``[[tool.mypy.overrides]]``. Keys follow the mypy reference, set-like arrays sort,
and ``plugins`` plus ``mypy_path`` retain input order.


**Top-level key ordering** (sectioned):

1. Import discovery: ``mypy_path`` → ``files`` → ``modules`` → ``packages`` → ``exclude`` →
   ``exclude_gitignore`` → ``namespace_packages`` → ``explicit_package_bases`` → ``ignore_missing_imports`` →
   ``follow_untyped_imports`` → ``follow_imports`` → ``follow_imports_for_stubs`` → ``python_executable`` →
   ``no_site_packages`` → ``no_silence_site_packages``
2. Platform configuration: ``python_version`` → ``platform`` → ``always_true`` → ``always_false``
3. Disallow dynamic typing: ``disallow_any_unimported`` → ``disallow_any_expr`` → ``disallow_any_decorated`` →
   ``disallow_any_explicit`` → ``disallow_any_generics`` → ``disallow_subclassing_any``
4. Untyped definitions and calls: ``disallow_untyped_calls`` → ``untyped_calls_exclude`` →
   ``disallow_untyped_defs`` → ``disallow_incomplete_defs`` → ``check_untyped_defs`` →
   ``disallow_untyped_decorators``
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
12. Report generation: ``any_exprs_report`` → ``cobertura_xml_report`` → ``html_report`` →
    ``linecount_report`` → ``linecoverage_report`` → ``lineprecision_report`` → ``txt_report`` →
    ``xml_report`` → ``xslt_html_report`` → ``xslt_txt_report``
13. Miscellaneous: ``junit_xml`` → ``junit_format`` → ``scripts_are_modules`` → ``warn_unused_configs`` →
    ``verbosity``
14. ``overrides`` last.

**Overrides entry key ordering:** in each ``[[tool.mypy.overrides]]`` entry, ``module`` comes first
(required), then per-module overridable keys in the same logical grouping as the parent table (import behavior,
platform markers, disallow dynamic typing, untyped defs/calls, optional handling, warnings, suppression,
miscellaneous strictness).

**Sorted arrays:**

- Top-level: ``files``, ``modules``, ``packages``, ``exclude``, ``always_true``, ``always_false``,
  ``untyped_calls_exclude``, ``deprecated_calls_exclude``, ``disable_error_code``, ``enable_error_code``.
- Inside overrides entries: ``module`` (when an array of patterns), ``always_true``, ``always_false``,
  ``disable_error_code``, ``enable_error_code``.

**Preserved order:** ``plugins`` (run in declared order; reordering changes behavior) and ``mypy_path``
(a search path with priority semantics).

**Inline-table handling:** when ``[[tool.mypy.overrides]]`` collapses to ``overrides = [{...}, {...}]`` under
the default ``table_format = "short"``, mypy-specific discriminators select each entry's key order:
``disable_error_code`` / ``enable_error_code`` / ``ignore_missing_imports`` / ``follow_untyped_imports`` /
``ignore_errors`` / ``warn_unused_ignores`` / ``disallow_untyped_defs`` / ``check_untyped_defs``. Arrays inside
each entry sort in either table shape.

.. code-block:: toml

   # Before
   [[tool.mypy.overrides]]
   disable_error_code = ["import-untyped", "attr-defined"]
   module = "pkg.*"

   # After
   [tool.mypy]
   overrides = [ { module = "pkg.*", disable_error_code = [ "attr-defined", "import-untyped" ] } ]

``[tool.pyrefly]``
~~~~~~~~~~~~~~~~~~

`Pyrefly <https://pyrefly.org/>`_ is Meta's Python type checker and language server, written in Rust. See its
`configuration reference <https://pyrefly.org/en/docs/configuration/>`_.

Keys follow platform → paths → behavior → ``errors``. Selection arrays sort; search paths retain input order.


**Key ordering:** ``python-version`` → ``python-platform`` → ``python-interpreter-path`` → ``project-includes`` →
``project-excludes`` → ``search-path`` → ``site-package-path`` → ``use-untyped-imports`` →
``replace-imports-with-any`` → ``ignore-errors-in-generated-code`` → ``errors``. Pyrefly spells its options with
hyphens; older underscore forms take the same rank.

**Sorted arrays:** ``project-includes``, ``project-excludes``. ``search-path`` and ``site-package-path`` keep
input order because pyrefly searches them in sequence. ``replace-imports-with-any`` also retains order because the
first match decides: a ``!`` rule exempts an import only while it precedes a broader matching rule.

``[tool.pyright]`` and ``[tool.basedpyright]``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

`Pyright <https://microsoft.github.io/pyright/>`_ is Microsoft's Python type checker;
`basedpyright <https://docs.basedpyright.com/>`_ is a community fork sharing the same schema. See the pyright
`configuration reference <https://microsoft.github.io/pyright/#/configuration>`_ and the basedpyright
`config-files reference <https://docs.basedpyright.com/latest/configuration/config-files/>`_.

Keys follow platform → mode flags → paths → strict-flavor toggles → ``defineConstant`` → alphabetical ``report*``
rules → ``executionEnvironments``. Path arrays sort.


**Key ordering:**

1. Platform / interpreter: ``pythonVersion`` → ``pythonPlatform`` → ``pythonPath`` → ``venv`` → ``venvPath``
   → ``typeshedPath`` → ``stubPath``
2. Mode flags: ``typeCheckingMode`` → ``strict`` → ``failOnWarnings`` → ``useLibraryCodeForTypes``
3. Paths: ``include`` → ``exclude`` → ``ignore`` → ``extraPaths``
4. Strict-flavor toggles: ``strictListInference``, ``strictDictionaryInference``, ``strictSetInference``,
   ``strictParameterNoneValue``, ``enableExperimentalFeatures``, ``enableTypeIgnoreComments``,
   ``analyzeUnannotatedFunctions``, ``disableBytesTypePromotions``, ``deprecateTypingAliases``
5. ``defineConstant``
6. All ``report*`` rules, alphabetized
7. ``executionEnvironments`` (last)

The formatter gathers ``report*`` rules from the input and sorts them, so new diagnostic names need no formatter
update.

**Sorted arrays:** ``include``, ``exclude``, ``ignore``, ``strict``. ``extraPaths`` keeps its order, since
pyright searches roots in list order.

``[tool.ty]``
~~~~~~~~~~~~~

`ty <https://docs.astral.sh/ty/>`_ is Astral's Python type checker, written in Rust. See its
`configuration reference <https://docs.astral.sh/ty/reference/configuration/>`_.

Keys follow ``src`` → ``environment`` → ``rules`` → ``terminal`` → ``overrides``. ``src.include`` sorts;
``src.exclude`` retains input order.


**Key ordering:** ``src`` → ``environment`` → ``rules`` → ``terminal`` → ``overrides`` (last). Within ``src``,
written either as dotted keys or as a ``[tool.ty.src]`` table: ``respect-ignore-files`` → ``include`` →
``exclude`` → ``exclude-scripts``.

**Sorted arrays:** ``src.include``. ``src.exclude`` keeps its order, since ty reads it the way a gitignore is
read and a ``!pattern`` takes back what a broader one excluded.

The schema remains pre-1.0; unknown keys follow the canonical set alphabetically.

``[tool.pytest.ini_options]``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

`pytest <https://docs.pytest.org/en/stable/>`_ is a testing framework for Python. See its
`configuration reference <https://docs.pytest.org/en/stable/reference/customize.html>`_.

Keys in ``ini_options`` follow the pytest reference. Set-like arrays sort; ``addopts`` and ``pythonpath`` retain input
order.


**Key ordering:** pytest itself → discovery → CLI arguments → markers/parametrize → warnings → doctest →
output → logging (capture / CLI / file) → JUnit XML → cache and tmp_path → assertion / faulthandler.

**Sorted arrays** (set semantics): ``norecursedirs``, ``collect_ignore``, ``collect_ignore_glob``,
``python_files``, ``python_classes``, ``python_functions``, ``markers``, ``doctest_optionflags``,
``usefixtures``, ``required_plugins``.

**Preserved order:** ``addopts`` (CLI arguments), ``testpaths`` (collection order),
``filterwarnings`` (the last filter that matches wins) and ``pythonpath`` (a search path with
priority semantics).

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

``[tool.coverage]``
~~~~~~~~~~~~~~~~~~~

`coverage.py <https://coverage.readthedocs.io/en/latest/>`_ measures code coverage of Python programs. See its
`configuration reference <https://coverage.readthedocs.io/en/latest/config.html>`_.

Keys follow coverage.py's workflow phases (run → paths → report → output formats) with related options kept adjacent;
set-like arrays sort.


**Key ordering:** coverage.py's workflow phases:

1. **Run phase** (``run.*``): data collection

   - Source selection: ``source`` → ``source_pkgs`` → ``source_dirs``
   - File filtering: ``include`` → ``omit``
   - Measurement: ``branch`` → ``cover_pylib`` → ``timid``
   - Execution context: ``command_line`` → ``concurrency`` → ``context`` → ``dynamic_context``
   - Data management: ``data_file`` → ``parallel`` → ``relative_files``
   - Extensions: ``plugins``
   - Debugging: ``debug`` → ``debug_file`` → ``disable_warnings``
   - Other: ``core`` → ``patch`` → ``sigterm``

2. **Paths** (``paths.*``): path mapping between source locations

3. **Report phase** (``report.*``): general reporting

   - Thresholds: ``fail_under`` → ``precision``
   - File filtering: ``include`` → ``omit`` → ``include_namespace_packages``
   - Line exclusion: ``exclude_lines`` → ``exclude_also``
   - Partial branches: ``partial_branches`` → ``partial_also``
   - Output control: ``skip_covered`` → ``skip_empty`` → ``show_missing``
   - Formatting: ``format`` → ``sort``
   - Error handling: ``ignore_errors``

4. **Output formats** (after report)

   - ``html.*``: ``directory`` → ``title`` → ``extra_css`` → ``show_contexts`` → ``skip_covered`` →
     ``skip_empty``
   - ``json.*``: ``output`` → ``pretty_print`` → ``show_contexts``
   - ``lcov.*``: ``output`` → ``line_checksums``
   - ``xml.*``: ``output`` → ``package_depth``

Related options stay adjacent: ``include`` / ``omit``, ``exclude_lines`` / ``exclude_also``,
``partial_branches`` / ``partial_also``, and ``skip_covered`` / ``skip_empty``.

**Sorted arrays:**

Run phase
  ``source``, ``source_pkgs``, ``source_dirs``, ``include``, ``omit``, ``concurrency``, ``plugins``, ``debug``,
  ``disable_warnings``

Report phase
  ``include``, ``omit``, ``exclude_lines``, ``exclude_also``, ``partial_branches``, ``partial_also``

.. code-block:: toml

   # Before
   [tool.coverage]
   report.exclude_also = ["if TYPE_CHECKING:"]
   report.omit = ["tests/*"]
   run.branch = true
   run.omit = ["tests/*"]

   # After
   [tool.coverage]
   run.omit = [ "tests/*" ]
   run.branch = true
   report.omit = [ "tests/*" ]
   report.exclude_also = [ "if TYPE_CHECKING:" ]

``[tool.tox]``
~~~~~~~~~~~~~~

`tox <https://tox.wiki/en/stable/>`_ automates and standardizes testing across multiple Python environments. See
its `configuration reference <https://tox.wiki/en/stable/config.html>`_.

A ``[tool.tox]`` block reuses the ``tox-toml-fmt`` rules applied to a standalone ``tox.toml``.


Reuses the rules from ``tox-toml-fmt``: alias normalization (``envlist`` → ``env_list``, ``setenv`` →
``set_env``, etc.), canonical key ordering for the root table and every env table, PEP 508 requirement
normalization and sorting in ``deps`` (``constraints`` retains file order), sorted ``pass_env`` (inline-table
entries first),
version-aware ``env_list`` sorting (``py313`` before ``py312`` before ``py311``), and inline-table reordering
for ``replace``, ``prefix``, ``product``, and ``value`` directives.

See the ``tox-toml-fmt`` documentation for the full schema and per-key behavior; the only difference here is
the namespace (``tool.tox`` instead of the root table).

``[tool.bumpversion]``
~~~~~~~~~~~~~~~~~~~~~~

`bump-my-version <https://callowayproject.github.io/bump-my-version/>`_ (the successor to bumpversion) updates
version strings across files and tags releases. See its `configuration reference
<https://callowayproject.github.io/bump-my-version/reference/configuration/>`_.

Keys follow identity → format → tag → commit → behavior → ``files`` / ``parts``.


**Key ordering:** identity (``current_version``) → format (``parse``, ``serialize``, ``search``, ``replace``,
``regex``, ``ignore_missing_*``) → tag (``tag``, ``sign_tags``, ``tag_name``, ``tag_message``) → commit
(``allow_dirty``, ``commit``, ``commit_args``, ``message``, ``moveable_tags``) → behavior → ``files`` /
``parts`` (arrays of tables, last).

``[tool.commitizen]``
~~~~~~~~~~~~~~~~~~~~~

`Commitizen <https://commitizen-tools.github.io/commitizen/>`_ enforces conventional commits and automates version
bumps and changelogs. See its
`configuration reference <https://commitizen-tools.github.io/commitizen/config/configuration_file/>`_.

Keys follow rule selection → version source → bump behavior → tag/sign → changelog → hooks → ``customize``.


**Key ordering:** rule selection (``name``, ``schema``, ``schema_pattern``, ``allowed_prefixes``) → version
source (``version``, ``version_scheme``, ``version_provider``, ``version_files``) → bump behavior → tag/sign →
changelog → hooks (``pre_bump_hooks``, ``post_bump_hooks``) → ``customize``.

**Sorted arrays:** ``version_files``, ``allowed_prefixes``, ``extras``, ``extra_files``.

``[tool.semantic_release]``
~~~~~~~~~~~~~~~~~~~~~~~~~~~

`python-semantic-release <https://python-semantic-release.readthedocs.io/en/latest/>`_ automates versioning and
releases from commit history. See its `configuration reference
<https://python-semantic-release.readthedocs.io/en/latest/configuration/configuration.html>`_.

Keys follow tag/version → assets → version source → repo → commit parser → branches → publish → changelog → remote.
``exclude_commit_patterns`` sorts; declaration lists retain input order.


**Key ordering:** tag/version → assets → version source → repo → commit parser → branches → publish →
changelog → remote.

**Sorted arrays:** ``exclude_commit_patterns``. ``version_variables``, ``version_toml`` and ``assets`` keep their
order: each declaration writes in turn and the later one decides what the file ends up holding.

``[tool.towncrier]``
~~~~~~~~~~~~~~~~~~~~

`towncrier <https://towncrier.readthedocs.io/en/stable/>`_ builds release notes from news-fragment files. See its
`configuration reference <https://towncrier.readthedocs.io/en/stable/configuration.html>`_.

Keys follow package identity → news location → rendering → behavior → ``type`` / ``section``. The ``ignore`` list
sorts; changelog entries retain display order.


**Key ordering:** package identity (``name``, ``version``, ``package``, ``package_dir``) → news location
(``directory``, ``filename``, ``start_string``, ``template``, ``title_format``, ``issue_format``,
``underlines``) → rendering (``wrap``, ``all_bullets``, ``single_file``, ``orphan_prefix``,
``create_eof_newline``, ``create_add_extension``) → behavior (``ignore``) → ``type`` and ``section`` (arrays
of tables, last).

``[[tool.towncrier.type]]`` entries get keys ordered ``directory`` → ``name`` → ``showcontent``;
``[[tool.towncrier.section]]`` entries get ``path`` → ``name`` → ``showcontent``. Arrays retain changelog display
order.

**Sorted arrays:** ``ignore`` (file globs to skip).

``[tool.pyproject-fmt]``
~~~~~~~~~~~~~~~~~~~~~~~~~

The formatter's own configuration table.

See the `configuration reference <https://pyproject-fmt.readthedocs.io/en/latest/configuration.html>`_ for what each key
controls.

Keys follow the documented configuration sequence. The ``expand_tables``, ``collapse_tables``, and
``skip_wrap_for_keys`` lists sort and drop duplicate strings.


**Key ordering:** ``column_width`` → ``indent`` → ``keep_full_version`` →
``generate_python_version_classifiers`` → ``max_supported_python`` → ``table_format`` → ``sub_table_spacing`` →
``separate_root_table`` → ``expand_tables`` → ``collapse_tables`` → ``skip_wrap_for_keys``. Unrecognized keys
follow alphabetically.

**Sorted arrays:** ``expand_tables``, ``collapse_tables``, ``skip_wrap_for_keys``. Matching treats each as a set,
so sorting and dropping byte-identical duplicates leaves behavior unchanged. Duplicate removal keeps case variants
distinct for case-sensitive lookup.

.. code-block:: toml

   # Before
   [tool.pyproject-fmt]
   keep_full_version = true
   column_width = 120
   skip_wrap_for_keys = ["b", "a", "a"]
   indent = 4

   # After
   [tool.pyproject-fmt]
   column_width = 120
   indent = 4
   keep_full_version = true
   skip_wrap_for_keys = [ "a", "b" ]

Other Tables
~~~~~~~~~~~~

Unrecognized tables take their standard table position. Their keys and values retain input order and spelling.
