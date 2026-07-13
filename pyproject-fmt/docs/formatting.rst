Formatting Rules
================

``pyproject-fmt`` is an opinionated formatter, much like `black <https://github.com/psf/black>`_ is for Python code. It
keeps configuration minimal so every ``pyproject.toml`` lands on one standard format. That buys you:

- less time configuring tools
- smaller diffs when committing changes
- code reviews where formatting never comes up

A few options exist (``column_width``, ``indent``, ``table_format``, ``sub_table_spacing``, ``separate_root_table``),
but there are no dozens of toggles. ``column_width`` controls when arrays split across lines and when string values
wrap with line continuations.

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

5. Any other tables (alphabetically)

String Quotes
~~~~~~~~~~~~~

All strings use double quotes by default. Single quotes are only used when the value contains double quotes:

.. fmt-example::

    name = 'my-package'
    description = "He said \"hello\""

Key Quotes
~~~~~~~~~~

TOML keys are normalized to the simplest valid form. Keys that are valid bare keys (containing only
``A-Za-z0-9_-``) have redundant quotes stripped. Single-quoted (literal) keys that require quoting are
converted to double-quoted (basic) strings with proper escaping. This applies to all keys: table headers,
key-value pairs, and inline table keys:

.. fmt-example::

    [tool."ruff"]
    "line-length" = 120
    lint.per-file-ignores.'tests/*' = ["S101"]

Backslashes and double quotes within literal keys are escaped during conversion:

.. fmt-example::

    lint.per-file-ignores.'path\to\file' = ["E501"]

Array Formatting
~~~~~~~~~~~~~~~~

Arrays are formatted based on line length, trailing comma presence, and comments. Short arrays stay on one line:

.. fmt-example::

    keywords = ["python", "toml"]

Arrays that exceed ``column_width`` are expanded and get a trailing comma (shown here with a small ``column_width`` to
keep the example short):

.. fmt-example::
    :config: column_width=30 generate_python_version_classifiers=false

    [project]
    keywords = ["web", "toml", "pyproject", "formatting"]

A trailing comma forces the multiline format, even for an array that would otherwise fit on one line:

.. fmt-example::

    classifiers = ["Development Status :: 4 - Beta",]

A comment on an entry also forces the multiline format. Here ``["E501", "E701"]`` would fit on one line, but the
comment keeps it expanded:

.. fmt-example::

    lint.ignore = [
      "E501", # too long
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
triple-quoted strings using line continuations (shown here with a small ``column_width``):

.. fmt-example::
    :config: column_width=40

    description = "Format your pyproject.toml file in place"

Wrapping prefers breaking at spaces and at ``" :: "`` separators (common in Python classifiers). Strings inside inline
tables are never wrapped. Strings that contain actual newlines are preserved as multi-line strings without adding line
continuations. Use ``skip_wrap_for_keys`` to prevent wrapping for specific keys.

Table Formatting
~~~~~~~~~~~~~~~~

Sub-tables can be formatted in two styles controlled by ``table_format``:

**Short format** (collapsed to dotted keys):

.. fmt-example::
    :config: generate_python_version_classifiers=false

    [project]
    urls.homepage = "https://example.com"
    urls.repository = "https://github.com/example/project"

**Long format** (expanded to table headers):

.. fmt-example::
    :config: table_format=long generate_python_version_classifiers=false

    [project.urls]
    homepage = "https://example.com"
    repository = "https://github.com/example/project"

**Table spacing:**

By default, different table groups (e.g. ``[project]`` and ``[tool.ruff]``) are separated by a blank line, while
sub-tables within the same group (e.g. ``[tool.ruff]`` and ``[tool.ruff.lint]``) are kept compact with no blank line
between them. You can control this with ``sub_table_spacing`` and ``separate_root_table``. Each option takes a string of
``\n`` characters where each ``\n`` adds one blank line. For example, setting ``sub_table_spacing = "\n"`` adds a blank
line between sub-tables:

.. fmt-example::
    :config: table_format=long sub_table_spacing=\n

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

.. fmt-example::

    lint.ignore = [
      "COM812", # Conflict with formatter
      "CPY", # No copyright statements
      "ISC001",   # Another rule
    ]

Disabled Keys
~~~~~~~~~~~~~

A commented-out line whose body is itself a single valid key-value (for example ``# default = true``) is treated as a
temporarily *disabled* field rather than free text. The formatter enables it for the duration of the pass, so it is laid
out and ordered together with the table it belongs to, then comments it out again on the way out. This keeps a disabled
key anchored to its entry instead of drifting to the next table, and formats the line the same way the enabled key would
be:

.. fmt-example::

    [[tool.uv.index]]
    name = "pypi"
    authenticate = "never"
    # default = true
    # ignore-error-codes = [400,401,403]

Comments that are not a single valid key-value (prose, multi-line blocks, commented-out table headers like
``# [tool.x]``) are left untouched and follow the usual comment-preservation rules above. The heuristic is purely
structural, so a prose comment that *happens* to be valid TOML (such as a ``key = value`` example written in
documentation) is reflowed too; if that matters, phrase the comment so it does not parse as a key-value. Keys that would
not fit on a single line within ``column_width`` are left as plain comments.

Group Markers
~~~~~~~~~~~~~

By default the formatter reorders each array, table, and section list as a single unit, so any entry can move to its
sorted position. Mark a boundary with a standalone comment that starts with ``# Group:``: the formatter then sorts within
each group, holds the groups in their original order, and keeps the marker at the top of its group. Reach for this when
related entries belong together but should still be sorted.

Files without a ``# Group:`` marker format the same as before, so the feature stays opt-in. Case does not matter, so
``# group:`` works too. Only standalone comment lines count; the formatter ignores inline trailing comments.

The formatter sorts the entries inside each group:

.. fmt-example::
    :config: generate_python_version_classifiers=false

    [project]
    dependencies = [
      # Group: web
      "flask",
      "django",
      # Group: db
      "sqlalchemy",
      "psycopg2",
    ]

A ``# Group:`` marker works the same way before a key in a table or before a ``[tool.*]`` header: the formatter sorts the
keys or sections up to the next marker, and never moves them across the boundary.

Table-Specific Handling
-----------------------

Beyond general formatting, each table has specific key ordering and value normalization rules.

``[build-system]``
~~~~~~~~~~~~~~~~~~

The :pep:`517` / :pep:`518` table that declares how your project is built. See the
`packaging specification <https://packaging.python.org/en/latest/specifications/pyproject-toml/#pyproject-build-system-table>`_.

Keys are ordered ``build-backend`` → ``requires`` → ``backend-path``, and ``requires`` is normalized and sorted. A
redundant ``wheel`` requirement is removed when the build backend is setuptools.

.. dropdown:: Formatting details

    **Key ordering:** ``build-backend`` → ``requires`` → ``backend-path``

    **Value normalization:**

    - ``requires``: dependencies normalized per :pep:`508` and sorted alphabetically by package name
    - ``backend-path``: entries sorted alphabetically

    **Redundant wheel removal:**

    A bare ``wheel`` entry is removed from ``requires`` when ``build-backend`` is ``setuptools.build_meta`` or
    ``setuptools.build_meta:__legacy__``: setuptools either injects ``wheel`` dynamically when building (before
    version 70.1) or bundles its own copy of ``bdist_wheel`` (70.1 and later), so listing it has no effect. The
    entry is kept when it carries a version constraint, extras, or markers (it then expresses intent the backend's
    dynamic injection would honor), when ``backend-path`` is set (an in-tree backend may import ``wheel``
    directly), or when ``setuptools`` itself is missing from ``requires``.

    .. fmt-example::

        [build-system]
        requires = ["setuptools >= 45", "wheel"]
        build-backend = "setuptools.build_meta"

``[project]``
~~~~~~~~~~~~~

The :pep:`621` core metadata table. See the
`packaging specification <https://packaging.python.org/en/latest/specifications/pyproject-toml/#pyproject-project-table>`_.

Keys follow the canonical metadata order; name, dependencies, classifiers, and keywords are normalized and sorted.

.. dropdown:: Formatting details

    **Key ordering:** ``name`` → ``version`` → ``import-names`` → ``import-namespaces`` → ``description`` →
    ``readme`` → ``keywords`` → ``license`` → ``license-files`` → ``maintainers`` → ``authors`` →
    ``requires-python`` → ``classifiers`` → ``dynamic`` → ``dependencies`` → ``optional-dependencies`` →
    ``urls`` → ``scripts`` → ``gui-scripts`` → ``entry-points``

    **Field normalizations:**

    ``name``
        Converted to canonical format (lowercase with hyphens): ``My_Package`` → ``my-package``

    ``description``
        Whitespace normalized: multiple spaces collapsed, consistent spacing after periods.

    ``license``
        License expression operators (``and``, ``or``, ``with``) uppercased: ``MIT or Apache-2.0`` →
        ``MIT OR Apache-2.0``

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

    **Dependency normalization:** every dependency array (``dependencies``, ``optional-dependencies.*``) is
    normalized per :pep:`508` (spaces removed, redundant ``.0`` suffixes stripped unless
    ``keep_full_version = true``) and sorted alphabetically by canonical package name:

    .. fmt-example::
        :config: generate_python_version_classifiers=false

        [project]
        dependencies = ["requests >= 2.0.0", "click~=8.0"]

    A direct-reference dependency keeps a space before its marker separator, because :pep:`508` only ends the URL
    at whitespace; without it, installers read the ``;`` and the marker as part of the URL and reject the entry:

    .. fmt-example::
        :config: generate_python_version_classifiers=false

        [project]
        dependencies = ["pkg @ git+https://github.com/user/repo.git@main; python_version>='3.10'"]

    **Optional-dependency extra names** are normalized to lowercase with hyphens:

    .. fmt-example::
        :config: generate_python_version_classifiers=false

        [project.optional-dependencies]
        Dev_Tools = ["pytest"]

    **Python version classifiers** are generated automatically from ``requires-python`` and
    ``max_supported_python`` (here ``3.14``). Disable with ``generate_python_version_classifiers = false``:

    .. fmt-example::

        [project]
        requires-python = ">=3.10"

    **Entry points:** inline tables within ``entry-points`` are expanded to dotted keys:

    .. fmt-example::
        :config: generate_python_version_classifiers=false

        [project]
        entry-points.console_scripts = { mycli = "mypackage:main" }

    **Authors / maintainers** can be inline tables (short format):

    .. fmt-example::
        :config: generate_python_version_classifiers=false

        [project]
        authors = [{ name = "Alice", email = "alice@example.com" }]

    or an expanded array of tables (long format, controlled by ``table_format``, ``expand_tables``, and
    ``collapse_tables``):

    .. fmt-example::
        :config: table_format=long generate_python_version_classifiers=false

        [[project.authors]]
        name = "Alice"
        email = "alice@example.com"

``[dependency-groups]``
~~~~~~~~~~~~~~~~~~~~~~~

The :pep:`735` table for named groups of development dependencies. See the
`packaging specification <https://packaging.python.org/en/latest/specifications/dependency-groups/>`_.

Groups are ordered ``dev`` → ``test`` → ``type`` → ``docs`` → others alphabetically; each group is normalized and sorted.

.. dropdown:: Formatting details

    **Key ordering:** ``dev`` → ``test`` → ``type`` → ``docs`` → others alphabetically

    **Value normalization:**

    - all dependencies normalized per :pep:`508`
    - sorted with regular dependencies first, then ``include-group`` entries

    .. fmt-example::

        [dependency-groups]
        dev = [{ include-group = "test" }, "ruff>=0.4", "mypy>=1"]

``[tool.poetry]``
~~~~~~~~~~~~~~~~~

`Poetry <https://python-poetry.org/>`_ is a Python dependency management and packaging tool. See its
`pyproject.toml reference <https://python-poetry.org/docs/pyproject/>`_.

Covers both Poetry 1.x (legacy metadata under ``[tool.poetry]``) and Poetry 2.x (metadata moved to ``[project]``,
Poetry-specific keys still here). Metadata is ordered by section, Poetry-specific inline tables get canonical key
order, and set-semantic arrays are sorted while order-significant ones are preserved.

.. dropdown:: Formatting details

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

    **Preserved as written** (order is semantically significant): ``authors``, ``maintainers``, ``packages``,
    ``include``, ``readme`` (when an array), multi-constraint dependency arrays, and ``[[tool.poetry.source]]``
    entries.

    **Inline-table key ordering:** when a Poetry-specific inline table is detected (via discriminator keys unique
    to Poetry's schema), its keys are reordered:

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

    Inline tables that don't match any Poetry-specific schema (for example ``[[project.authors]]`` inline form
    ``{ name = "...", email = "..." }``) are left untouched.

    .. fmt-example::

        [[tool.poetry.source]]
        priority = "primary"
        url = "https://example.com"
        name = "private"

        [tool.poetry.dependencies]
        zebra = "^1.0"
        python = "^3.11"
        foo = { branch = "main", git = "https://example.com/foo" }

``[tool.pdm.*]``
~~~~~~~~~~~~~~~~

`PDM <https://pdm-project.org/latest/>`_ is a modern Python package and dependency manager. See its
`build configuration reference <https://pdm-project.org/latest/reference/build/>`_.

Top-level keys are ordered distribution → resolution → version → build → scripts → source → dev-dependencies →
publish → options; name and glob arrays are sorted, while source-entry order is preserved.

.. dropdown:: Formatting details

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

`setuptools <https://setuptools.pypa.io/en/latest/>`_ is a long-standing build backend and packaging library;
`setuptools_scm <https://setuptools-scm.readthedocs.io/en/latest/>`_ derives the package version from SCM tags. See
the setuptools `pyproject.toml reference <https://setuptools.pypa.io/en/latest/userguide/pyproject_config.html>`_
and the setuptools_scm `configuration reference <https://setuptools-scm.readthedocs.io/en/latest/config/>`_.

Keys in both tables are grouped (discovery → data → metadata → deprecated); name and glob arrays are sorted, while
literal lists like ``packages`` are preserved.

.. dropdown:: Formatting details

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
    value (an array of glob patterns) is sorted alphabetically.

    ``[tool.setuptools.dynamic]`` ordering: field names alphabetized. Inline-table directives (e.g.
    ``version = { attr = "pkg.__version__" }`` or ``readme = { file = "README.md", content-type = "text/markdown" }``)
    get their keys ordered ``attr`` → ``file`` → ``content-type``.

    **Sorted arrays:**

    - ``py-modules``, ``platforms``, ``provides``, ``obsoletes``, ``script-files``, ``namespace-packages``,
      ``eager-resources``: alphabetized.
    - ``packages.find.include`` / ``packages.find.exclude`` / ``packages.find-namespace.*``: alphabetized.
    - Values inside ``package-data`` / ``exclude-package-data`` / ``data-files`` tables: alphabetized.

    **Preserved as written** (order is meaningful): ``packages`` (literal list, first match wins),
    ``license-files`` (PEP 639 concatenation order), and everything under ``[[tool.setuptools.ext-modules]]``
    (compiler and linker argv arrays).

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

    .. fmt-example::

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

``[tool.hatch.*]``
~~~~~~~~~~~~~~~~~~

`Hatch <https://hatch.pypa.io/latest/>`_ is a modern, extensible Python project manager built around the Hatchling
build backend. See its `build configuration reference <https://hatch.pypa.io/latest/config/build/>`_.

Keys across the many ``[tool.hatch.*]`` sub-tables are grouped (version → metadata → build → publish → workspace →
environments); name and path arrays are sorted, while build-hook and matrix order are preserved.

.. dropdown:: Formatting details

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

    - Build: ``include``, ``exclude``, ``force-include``, ``artifacts``, ``packages``, ``sources``,
      ``dev-mode-dirs``, and the matching ``build.targets.wheel.*`` / ``build.targets.sdist.*`` arrays.
    - Environments: per-env ``dependencies``, ``extra-dependencies``, ``features``, ``platforms``,
      ``env-include``, ``env-exclude``, ``pre-install-commands``, ``post-install-commands``.
    - Workspace: ``members``, ``exclude``.

    ``scripts`` and ``env-vars`` sub-tables under each environment have their inner keys alphabetized.

    **Preserved as written:** build-hook order and matrix entry order (both carry semantic meaning).

``[tool.scikit-build]``
~~~~~~~~~~~~~~~~~~~~~~~

`scikit-build-core <https://scikit-build-core.readthedocs.io/en/latest/>`_ is a CMake-based build backend for
Python C/C++ extensions. See its `configuration reference
<https://scikit-build-core.readthedocs.io/en/latest/configuration/index.html>`_.

Keys are ordered meta → build → cmake → ninja → sdist → wheel → install → editable → logging → metadata → search
→ ``generate`` → ``overrides``; name and path lists are sorted, while cmake/ninja argv are preserved.

.. dropdown:: Formatting details

    **Key ordering:** meta keys (``minimum-version``, ``build-dir``, ``fail``, ``experimental``,
    ``strict-config``) → ``build`` → ``cmake`` → ``ninja`` → ``sdist`` → ``wheel`` → ``install`` → ``editable`` →
    ``logging`` / ``messages`` → ``metadata`` → ``search`` → ``generate`` (array of tables) → ``overrides``
    (array of tables).

    **Sorted arrays:** ``include``, ``exclude``, ``packages``, ``files``, ``targets``, ``components``,
    ``exclude-fields``.

    **Preserved as written:** ``args`` and ``define`` (CLI argv for cmake/ninja).

``[tool.maturin]``
~~~~~~~~~~~~~~~~~~

`Maturin <https://www.maturin.rs/>`_ builds and publishes Rust-based Python extension modules. See its
`configuration reference <https://www.maturin.rs/config>`_.

Keys are ordered module identity → source layout → cargo settings → compatibility/strip → behavior; set-semantic
arrays are sorted, while cargo/rustc argv are preserved.

.. dropdown:: Formatting details

    **Key ordering:** module identity (``module-name``, ``bindings``, ``python-source``, ``python-packages``,
    ``python-bin-path``) → source layout (``src``, ``manifest-path``, ``include``, ``exclude``, ``sdist-include``,
    ``sdist-generator``, ``data``) → cargo settings (``features``, ``no-default-features``, ``all-features``,
    ``cargo-extra-args``, ``rustc-extra-args``, ``config``, ``profile``, ``target``, ``target-dir``) →
    compatibility / strip (``compatibility``, ``auditwheel``, ``skip-auditwheel``, ``strip``, ``frozen``,
    ``locked``, ``offline``, ``zig``) → behavior (``use-cross``).

    **Sorted arrays:** ``python-packages``, ``include``, ``exclude``, ``sdist-include``, ``features`` (all set
    semantics).

    **Preserved as written:** ``cargo-extra-args`` / ``rustc-extra-args`` (CLI argv).

``[tool.pixi]``
~~~~~~~~~~~~~~~

`Pixi <https://pixi.prefix.dev/latest/>`_ is a cross-platform conda/PyPI package and environment manager. See its
`pyproject.toml reference <https://pixi.prefix.dev/latest/python/pyproject_toml/>`_.

Keys are grouped by function (workspace metadata → configuration → dependencies → environments → build); channel and
platform arrays are sorted.

.. dropdown:: Formatting details

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

    **Sorted arrays:** ``workspace.channels``, ``workspace.platforms``, ``workspace.preview``,
    ``workspace.build-variants-files``.

``[tool.uv]``
~~~~~~~~~~~~~

`uv <https://docs.astral.sh/uv/>`_ is a fast Python package and project manager from Astral. See its
`settings reference <https://docs.astral.sh/uv/reference/settings/>`_.

Keys are grouped by function (Python → dependencies → sources → resolution → build → network → publishing →
workspace); package-name arrays and the ``sources`` table are sorted alphabetically.

.. dropdown:: Formatting details

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

    **Sources table:** ``sources`` entries are sorted alphabetically by package name:

    .. fmt-example::

        [tool.uv.sources]
        zebra = { git = "..." }
        alpha = { path = "..." }

    **pip subsection:** ``[tool.uv.pip]`` follows the same rules, with arrays like ``extra``, ``no-binary-package``,
    ``no-build-package``, ``reinstall-package``, and ``upgrade-package`` sorted alphabetically.

``[tool.cibuildwheel]``
~~~~~~~~~~~~~~~~~~~~~~~

`cibuildwheel <https://cibuildwheel.pypa.io/en/stable/>`_ builds Python wheels across platforms in CI. See its
`options reference <https://cibuildwheel.pypa.io/en/stable/options/>`_.

Keys are ordered selection → build config → build phases → test phases → platform images → per-platform sub-tables
→ ``overrides``; set-semantic arrays are sorted, while argv-like lists are preserved.

.. dropdown:: Formatting details

    **Key ordering:** selection (``build``, ``skip``, ``test-skip``, ``archs``, ``enable``,
    ``free-threaded-support``) → build configuration (``build-frontend``, ``build-verbosity``, ``config-settings``,
    ``dependency-versions``, ``environment``, ``environment-pass``) → build phases (``before-all``,
    ``before-build``, ``repair-wheel-command``) → test phases (``before-test``, ``test-command``,
    ``test-requires``, ``test-extras``, ``test-groups``, ``test-sources``) → platform images
    (``manylinux-*-image``, ``musllinux-*-image``) → ``container-engine`` → per-platform sub-tables (``linux``,
    ``macos``, ``windows``, ``android``, ``ios``, ``pyodide``) → ``overrides`` last.

    Per-platform sub-tables follow the same inner ordering. ``[[tool.cibuildwheel.overrides]]`` entries place
    ``select`` first (required), then the regular cibuildwheel keys; the array order itself is preserved (later
    overrides win).

    **Sorted arrays:** ``enable``, ``test-extras``, ``test-groups``.

    **Preserved as written:** most other array-valued keys (``test-requires``, ``before-all``, ``test-command``,
    the various ``environment*`` fields) are CLI argv or ordered lists.

``[tool.autopep8]``
~~~~~~~~~~~~~~~~~~~

`autopep8 <https://github.com/hhatto/autopep8>`_ automatically formats Python code to conform to PEP 8. See its
`configuration reference <https://github.com/hhatto/autopep8#pyproject-toml>`_.

Keys are ordered length/indent → mode → rules → behavior; rule lists are sorted.

.. dropdown:: Formatting details

    **Key ordering:** length/indent → mode (``in-place``, ``recursive``, ``diff``, ``list-fixes``) → rules
    (``ignore``, ``select``, ``exclude``) → behavior.

    **Sorted arrays:** ``ignore``, ``select``, ``exclude``.

``[tool.black]``
~~~~~~~~~~~~~~~~

`Black <https://black.readthedocs.io/en/stable/>`_ is an opinionated Python code formatter. See its
`configuration reference <https://black.readthedocs.io/en/stable/usage_and_configuration/the_basics.html>`_.

Keys follow Black's option grouping; ``target-version`` and ``enable-unstable-feature`` arrays are alphabetized.

.. dropdown:: Formatting details

    **Key ordering:**

    1. ``required-version`` → ``target-version`` → ``line-length``
    2. File selection: ``include`` → ``extend-exclude`` → ``force-exclude`` → ``exclude``
    3. Behavior: ``skip-string-normalization`` → ``skip-magic-trailing-comma`` → ``preview`` → ``unstable`` →
       ``enable-unstable-feature`` → ``fast`` → ``workers``
    4. Output: ``color`` → ``verbose`` → ``quiet``

    **Sorted arrays:** ``target-version`` (so ``py39`` precedes ``py310``), ``enable-unstable-feature``.

    The ``include`` / ``exclude`` family are regex strings, not arrays, so they're left as-is.

``[tool.yapf]``
~~~~~~~~~~~~~~~

`YAPF <https://github.com/google/yapf>`_ is a configurable Python code formatter from Google. See its
`configuration reference <https://github.com/google/yapf#knobs>`_.

A single flat table: ``based_on_style`` comes first (it sets the defaults), then the rest in a fixed order.

.. dropdown:: Formatting details

    **Key ordering:** ``based_on_style`` first (sets defaults), then ``column_limit``, ``indent_width``,
    ``continuation_indent_width``, then the remaining keys alphabetized.

``[tool.djlint]``
~~~~~~~~~~~~~~~~~

`djLint <https://djlint.com/>`_ is a linter and formatter for HTML templates (Django, Jinja, and more). See its
`configuration reference <https://djlint.com/docs/configuration/>`_.

Keys are ordered profile/scope → formatting → linting → ignores → output; exclude and block lists are sorted.

.. dropdown:: Formatting details

    **Key ordering:** profile/scope → formatting → linting → ignores → output.

    **Sorted arrays:** ``exclude``, ``extend_exclude``, ``custom_blocks``, ``custom_html``, ``ignore``,
    ``ignore_blocks``.

``[tool.ruff]``
~~~~~~~~~~~~~~~

`Ruff <https://docs.astral.sh/ruff/>`_ is a fast Python linter and formatter written in Rust. See its
`settings reference <https://docs.astral.sh/ruff/settings/>`_.

Keys follow Ruff's option grouping (global → paths → behavior → output → ``format`` → ``lint``); rule-code, path, and
name arrays are sorted with natural ordering (``RUF1`` < ``RUF9`` < ``RUF10``).

.. dropdown:: Formatting details

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
    are sorted too:

    .. fmt-example::

        [tool.ruff]
        lint.select = ["F", "E", "RUF", "I"]
        lint.ignore = ["E701", "E501"]
        lint.per-file-ignores."tests/*.py" = ["S101", "D103"]

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
      ``lint.isort.default-section``, ``lint.isort.extra-standard-library``, ``lint.isort.forced-separate``,
      ``lint.isort.no-lines-before``, ``lint.isort.required-imports``, ``lint.isort.single-line-exclusions``,
      ``lint.isort.variables``, ``lint.pep8-naming.classmethod-decorators``,
      ``lint.pep8-naming.extend-ignore-names``, ``lint.pep8-naming.ignore-names``,
      ``lint.pep8-naming.staticmethod-decorators``, ``lint.pydocstyle.ignore-decorators``,
      ``lint.pydocstyle.property-decorators``, ``lint.pyflakes.extend-generics``,
      ``lint.pylint.allow-dunder-method-names``, ``lint.pylint.allow-magic-value-types``

``[tool.isort]``
~~~~~~~~~~~~~~~~

`isort <https://pycqa.github.io/isort/>`_ sorts and organizes Python imports. See its
`configuration options <https://pycqa.github.io/isort/docs/configuration/options.html>`_.

``profile`` comes first (it sets the defaults everything else overrides), then output style, known sources,
separation, skip patterns, and import edits; name lists are sorted, while sequence-sensitive lists are preserved.

.. dropdown:: Formatting details

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
    ``blocked_extensions``, ``single_line_exclusions``, ``forced_separate``, ``treat_comments_as_code``,
    ``treat_all_comments_as_code``, ``constants``, ``variables``.

    **Preserved as written** (sequence is significant): ``sections`` (output section order), ``no_lines_before``,
    ``add_imports``, ``remove_imports``, ``required_imports``, ``force_to_top``.

``[tool.pylint.*]``
~~~~~~~~~~~~~~~~~~~

`Pylint <https://pylint.readthedocs.io/en/stable/>`_ is a static analyzer and linter for Python. See
its `configuration reference <https://pylint.readthedocs.io/en/stable/user_guide/configuration/index.html>`_.

Sub-tables follow Pylint's checker-group order; all rule, name, and path lists are sorted by leaf key name
regardless of sub-table.

.. dropdown:: Formatting details

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

Keys are ordered dictionaries → scope → fix behavior → output; word and path lists are sorted.

.. dropdown:: Formatting details

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

Keys are ordered behavior → format width → wrap/summary tweaks → other.

.. dropdown:: Formatting details

    **Key ordering:** behavior (``in-place``, ``recursive``, ``check``, ``diff``, ``black``, ``pep257``,
    ``non-strict``) → format width (``line-length``, ``wrap-summaries``, ``wrap-descriptions``, ``tab-width``) →
    wrap/summary tweaks → other.

``[tool.interrogate]``
~~~~~~~~~~~~~~~~~~~~~~

`interrogate <https://interrogate.readthedocs.io/en/latest/>`_ measures docstring coverage of a Python codebase.
See its `configuration reference <https://interrogate.readthedocs.io/en/latest/#configuration>`_.

Keys are ordered threshold → ignore flags → exclude → output; exclude and regex lists are sorted.

.. dropdown:: Formatting details

    **Key ordering:** threshold → ignore flags → exclude → output.

    **Sorted arrays:** ``exclude``, ``extend-exclude``, ``ignore-regex``.

``[tool.check-manifest]``
~~~~~~~~~~~~~~~~~~~~~~~~~

`check-manifest <https://github.com/mgedmin/check-manifest>`_ checks that ``MANIFEST.in`` is complete for an
sdist. See its `configuration reference <https://github.com/mgedmin/check-manifest#configuration>`_.

Keys are ordered ``ignore`` → ``ignore-bad-ideas`` → ``ignore-default-rules``; both glob lists are sorted.

.. dropdown:: Formatting details

    **Key ordering:** ``ignore`` → ``ignore-bad-ideas`` → ``ignore-default-rules``.

    **Sorted arrays:** ``ignore`` and ``ignore-bad-ideas`` (file-glob lists).

``[tool.deptry]``
~~~~~~~~~~~~~~~~~

`deptry <https://deptry.com/>`_ finds unused, missing, and transitive dependencies in Python projects. See its
`usage reference <https://deptry.com/usage/>`_.

Keys are ordered scope/exclude → ignore rules → per-rule ignores → behavior → mapping; the ignore and path lists
are sorted.

.. dropdown:: Formatting details

    **Key ordering:** scope/exclude → ignore rules → per-rule ignores → behavior → mapping.

    **Sorted arrays:** the ``ignore_*`` / ``exclude`` / ``requirements_files`` / ``pep621_dev_dependency_groups``
    / ``known_first_party`` lists.

``[tool.vulture]``
~~~~~~~~~~~~~~~~~~

`Vulture <https://github.com/jendrikseipp/vulture>`_ finds unused (dead) Python code. See its
`configuration reference <https://github.com/jendrikseipp/vulture#configuration>`_.

Keys are ordered paths → ignore → behavior → output; path and name lists are sorted.

.. dropdown:: Formatting details

    **Key ordering:** paths → ignore (``exclude``, ``ignore_names``, ``ignore_decorators``) → behavior
    (``make_whitelist``, ``min_confidence``, ``sort_by_size``) → output (``verbose``).

    **Sorted arrays:** ``paths``, ``exclude``, ``ignore_names``, ``ignore_decorators``.

``[tool.bandit]``
~~~~~~~~~~~~~~~~~

`Bandit <https://bandit.readthedocs.io/en/latest/>`_ finds common security issues in Python code. See its
`configuration reference <https://bandit.readthedocs.io/en/latest/config.html>`_.

Keys are ordered ``exclude_dirs`` → ``targets`` → ``tests`` → ``skips`` → per-plugin sub-tables; all array values
are alphabetized.

.. dropdown:: Formatting details

    **Key ordering:** ``exclude_dirs`` → ``targets`` → ``tests`` → ``skips`` → per-plugin sub-tables
    (``assert_used``, ``hardcoded_tmp_directory``, etc.).

    **Sorted arrays:** all array values (rule IDs, directory paths, function-name lists, all set semantics).

``[tool.mypy]``
~~~~~~~~~~~~~~~

`mypy <https://mypy.readthedocs.io/en/stable/>`_ is a static type checker for Python. See its
`configuration reference <https://mypy.readthedocs.io/en/stable/config_file.html>`_.

Covers all documented mypy options plus the ``[[tool.mypy.overrides]]`` array of tables, reordered to match mypy's
configuration reference; set-semantic arrays are sorted, while ``plugins`` and ``mypy_path`` are preserved.

.. dropdown:: Formatting details

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

    **Preserved as written:** ``plugins`` (run in declared order; reordering changes behavior) and ``mypy_path``
    (a search path with priority semantics).

    **Inline-table handling:** when ``[[tool.mypy.overrides]]`` collapses to ``overrides = [{...}, {...}]`` under
    the default ``table_format = "short"``, key order inside each entry is normalized via discriminators unique to
    mypy (``disable_error_code`` / ``enable_error_code`` / ``ignore_missing_imports`` / ``follow_untyped_imports``
    / ``ignore_errors`` / ``warn_unused_ignores`` / ``disallow_untyped_defs`` / ``check_untyped_defs``). The arrays
    inside each inline entry are sorted in place, so ``disable_error_code = [...]`` is alphabetized whether the
    override is expanded or collapsed.

    .. fmt-example::

        [[tool.mypy.overrides]]
        disable_error_code = ["import-untyped", "attr-defined"]
        module = "pkg.*"

``[tool.pyrefly]``
~~~~~~~~~~~~~~~~~~

`Pyrefly <https://pyrefly.org/>`_ is Meta's fast Python type checker and language server, written in Rust. See its
`configuration reference <https://pyrefly.org/en/docs/configuration/>`_.

Keys follow a fixed platform → paths → behavior → ``errors`` order; path arrays are sorted.

.. dropdown:: Formatting details

    **Key ordering:** ``python_version`` → ``python_platform`` → ``python_interpreter`` → ``project_includes`` →
    ``project_excludes`` → ``search_path`` → ``site_package_path`` → ``use_untyped_imports`` →
    ``replace_imports_with_any`` → ``ignore_errors_in_generated_code`` → ``errors``.

    **Sorted arrays:** the path arrays.

``[tool.pyright]`` and ``[tool.basedpyright]``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

`Pyright <https://microsoft.github.io/pyright/>`_ is Microsoft's fast Python type checker;
`basedpyright <https://docs.basedpyright.com/>`_ is a community fork sharing the same schema. See the pyright
`configuration reference <https://microsoft.github.io/pyright/#/configuration>`_ and the basedpyright
`config-files reference <https://docs.basedpyright.com/latest/configuration/config-files/>`_.

Keys are ordered platform → mode flags → paths → strict-flavor toggles → ``defineConstant`` → ``report*`` rules
(alphabetized) → ``executionEnvironments``; path arrays are sorted.

.. dropdown:: Formatting details

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

    The ``report*`` rules (70+ in pyright; basedpyright adds more) are collected from the input and inserted
    alphabetically rather than hardcoded, so new diagnostic rules don't require formatter changes.

    **Sorted arrays:** ``include``, ``exclude``, ``ignore``, ``extraPaths``, ``strict``.

``[tool.ty]``
~~~~~~~~~~~~~

`ty <https://docs.astral.sh/ty/>`_ is Astral's fast Python type checker, written in Rust. See its
`configuration reference <https://docs.astral.sh/ty/reference/configuration/>`_.

Keys are ordered ``src`` → ``respect-ignore-files`` → ``environment`` → ``rules`` → ``terminal`` → ``overrides``;
the ``src`` array is sorted.

.. dropdown:: Formatting details

    **Key ordering:** ``src`` → ``respect-ignore-files`` → ``environment`` → ``rules`` → ``terminal`` →
    ``overrides`` (last).

    **Sorted arrays:** ``src``.

    The schema is still pre-1.0; unknown keys are alphabetized after the canonical set.

``[tool.pytest.ini_options]``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

`pytest <https://docs.pytest.org/en/stable/>`_ is a feature-rich testing framework for Python. See its
`configuration reference <https://docs.pytest.org/en/stable/reference/customize.html>`_.

Keys in the ``ini_options`` block follow the pytest reference order; set-semantic arrays are sorted, while
``addopts`` and ``pythonpath`` are preserved.

.. dropdown:: Formatting details

    **Key ordering:** pytest itself → discovery → CLI arguments → markers/parametrize → warnings → doctest →
    output → logging (capture / CLI / file) → JUnit XML → cache and tmp_path → assertion / faulthandler.

    **Sorted arrays** (set semantics): ``testpaths``, ``norecursedirs``, ``collect_ignore``,
    ``collect_ignore_glob``, ``python_files``, ``python_classes``, ``python_functions``, ``markers``,
    ``filterwarnings``, ``doctest_optionflags``, ``usefixtures``, ``required_plugins``.

    **Preserved as written:** ``addopts`` (CLI argv, order matters) and ``pythonpath`` (a search path with
    priority semantics).

    .. fmt-example::

        [tool.pytest.ini_options]
        log_cli_level = "INFO"
        markers = [ "slow: marks tests as slow", "fast: marks tests as fast" ]
        addopts = [ "--strict-markers", "-ra" ]
        testpaths = [ "tests" ]
        minversion = "8"

``[tool.coverage]``
~~~~~~~~~~~~~~~~~~~

`coverage.py <https://coverage.readthedocs.io/en/latest/>`_ measures code coverage of Python programs. See its
`configuration reference <https://coverage.readthedocs.io/en/latest/config.html>`_.

Keys follow coverage.py's workflow phases (run → paths → report → output formats) with related options kept adjacent;
set-semantic arrays are sorted.

.. dropdown:: Formatting details

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

    .. fmt-example::

        [tool.coverage]
        report.exclude_also = ["if TYPE_CHECKING:"]
        report.omit = ["tests/*"]
        run.branch = true
        run.omit = ["tests/*"]

``[tool.tox]``
~~~~~~~~~~~~~~

`tox <https://tox.wiki/en/stable/>`_ automates and standardizes testing across multiple Python environments. See
its `configuration reference <https://tox.wiki/en/stable/config.html>`_.

A ``[tool.tox]`` block in ``pyproject.toml`` reuses the ``tox-toml-fmt`` rules, so it is formatted identically to a
standalone ``tox.toml``.

.. dropdown:: Formatting details

    Reuses the rules from ``tox-toml-fmt``: alias normalization (``envlist`` → ``env_list``, ``setenv`` →
    ``set_env``, etc.), canonical key ordering for the root table and every env table, PEP 508 requirement
    normalization and sorting in ``deps`` and ``constraints``, sorted ``pass_env`` (inline-table entries first),
    version-aware ``env_list`` sorting (``py313`` before ``py312`` before ``py311``), and inline-table reordering
    for ``replace``, ``prefix``, ``product``, and ``value`` directives.

    See the ``tox-toml-fmt`` documentation for the full schema and per-key behavior; the only difference here is
    the namespace (``tool.tox`` instead of the root table).

``[tool.bumpversion]``
~~~~~~~~~~~~~~~~~~~~~~

`bump-my-version <https://callowayproject.github.io/bump-my-version/>`_ (the successor to bumpversion) updates
version strings across files and tags releases. See its `configuration reference
<https://callowayproject.github.io/bump-my-version/reference/configuration/>`_.

Keys are ordered identity → format → tag → commit → behavior → ``files`` / ``parts``.

.. dropdown:: Formatting details

    **Key ordering:** identity (``current_version``) → format (``parse``, ``serialize``, ``search``, ``replace``,
    ``regex``, ``ignore_missing_*``) → tag (``tag``, ``sign_tags``, ``tag_name``, ``tag_message``) → commit
    (``allow_dirty``, ``commit``, ``commit_args``, ``message``, ``moveable_tags``) → behavior → ``files`` /
    ``parts`` (arrays of tables, last).

``[tool.commitizen]``
~~~~~~~~~~~~~~~~~~~~~

`Commitizen <https://commitizen-tools.github.io/commitizen/>`_ enforces conventional commits and automates version
bumps and changelogs. See its
`configuration reference <https://commitizen-tools.github.io/commitizen/config/configuration_file/>`_.

Keys are ordered rule selection → version source → bump behavior → tag/sign → changelog → hooks → ``customize``.

.. dropdown:: Formatting details

    **Key ordering:** rule selection (``name``, ``schema``, ``schema_pattern``, ``allowed_prefixes``) → version
    source (``version``, ``version_scheme``, ``version_provider``, ``version_files``) → bump behavior → tag/sign →
    changelog → hooks (``pre_bump_hooks``, ``post_bump_hooks``) → ``customize``.

    **Sorted arrays:** ``version_files``, ``allowed_prefixes``, ``extras``, ``extra_files``.

``[tool.semantic_release]``
~~~~~~~~~~~~~~~~~~~~~~~~~~~

`python-semantic-release <https://python-semantic-release.readthedocs.io/en/latest/>`_ automates versioning and
releases from commit history. See its `configuration reference
<https://python-semantic-release.readthedocs.io/en/latest/configuration/configuration.html>`_.

Keys are ordered tag/version → assets → version source → repo → commit parser → branches → publish → changelog →
remote; version and asset lists are sorted.

.. dropdown:: Formatting details

    **Key ordering:** tag/version → assets → version source → repo → commit parser → branches → publish →
    changelog → remote.

    **Sorted arrays:** ``version_variables``, ``version_toml``, ``assets``, ``exclude_commit_patterns``.

``[tool.towncrier]``
~~~~~~~~~~~~~~~~~~~~

`towncrier <https://towncrier.readthedocs.io/en/stable/>`_ builds release notes from news-fragment files. See its
`configuration reference <https://towncrier.readthedocs.io/en/stable/configuration.html>`_.

Keys are ordered package identity → news location → rendering → behavior → ``type`` / ``section``; the ``ignore``
glob list is sorted, while changelog display order is preserved.

.. dropdown:: Formatting details

    **Key ordering:** package identity (``name``, ``version``, ``package``, ``package_dir``) → news location
    (``directory``, ``filename``, ``start_string``, ``template``, ``title_format``, ``issue_format``,
    ``underlines``) → rendering (``wrap``, ``all_bullets``, ``single_file``, ``orphan_prefix``,
    ``create_eof_newline``, ``create_add_extension``) → behavior (``ignore``) → ``type`` and ``section`` (arrays
    of tables, last).

    ``[[tool.towncrier.type]]`` entries get keys ordered ``directory`` → ``name`` → ``showcontent``;
    ``[[tool.towncrier.section]]`` entries get ``path`` → ``name`` → ``showcontent``. Array order is preserved
    (display order in the rendered changelog).

    **Sorted arrays:** ``ignore`` (file globs to skip).

Other Tables
~~~~~~~~~~~~

Any unrecognized tables are preserved and reordered according to standard table ordering rules. Keys within unknown
tables are not reordered or normalized.
