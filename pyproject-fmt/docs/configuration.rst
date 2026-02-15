Configuration
=============

Configuration via file
----------------------

The ``tool.pyproject-fmt`` table is used when present in the ``pyproject.toml`` file:

.. code-block:: toml

    [tool.pyproject-fmt]

    # After how many columns split arrays/dicts into multiple lines and wrap long strings;
    # use a trailing comma in arrays to force multiline format instead of lowering this value
    column_width = 120

    # Number of spaces for indentation
    indent = 2

    # Keep full version numbers (e.g., 1.0.0 instead of 1.0) in dependency specifiers
    keep_full_version = false

    # Automatically generate Python version classifiers based on requires-python
    # Set to false to disable automatic classifier generation
    generate_python_version_classifiers = true

    # Maximum Python version for generating version classifiers
    max_supported_python = "3.14"

    # Table format: "short" collapses sub-tables to dotted keys, "long" expands to [table.subtable] headers
    table_format = "short"

    # List of tables to force expand regardless of table_format setting
    expand_tables = []

    # List of tables to force collapse regardless of table_format or expand_tables settings
    collapse_tables = []

    # List of key patterns to skip string wrapping (supports wildcards like *.parse or tool.bumpversion.*)
    skip_wrap_for_keys = []

If not set they will default to values from the CLI.

Command line interface
----------------------

.. sphinx_argparse_cli::
    :module: pyproject_fmt.__main__
    :func: _build_our_cli
    :prog: pyproject-fmt
    :title:

Python version classifiers
--------------------------

This tool will automatically generate the ``Programming Language :: Python :: 3.X`` classifiers for you. To do so it
needs to know the range of Python interpreter versions you support:

- The lower bound can be set via the ``requires-python`` key in the ``pyproject.toml`` configuration file (defaults to
  the oldest non end of line CPython at the time of the release).
- The upper bound, by default, will assume the latest stable release of CPython at the time of the release, but can be
  changed via CLI flag or the config file.

Table formatting
----------------

.. note::

    Table formatting options are available in version 2.12.0 and later.

You can control how sub-tables are formatted in your ``pyproject.toml`` file. There are two formatting styles:

**Short format (collapsed)** - The default behavior where sub-tables are collapsed into dotted keys. Use this for a
compact representation:

.. code-block:: toml

    [project]
    name = "myproject"
    urls.homepage = "https://example.com"
    urls.repository = "https://github.com/example/myproject"
    scripts.mycli = "mypackage:main"

**Long format (expanded)** - Sub-tables are expanded into separate ``[table.subtable]`` sections. Use this for
readability when tables have many keys or complex values:

.. code-block:: toml

    [project]
    name = "myproject"

    [project.urls]
    homepage = "https://example.com"
    repository = "https://github.com/example/myproject"

    [project.scripts]
    mycli = "mypackage:main"

Configuration priority
~~~~~~~~~~~~~~~~~~~~~~

The formatting behavior is determined by a priority system that allows you to set a global default while overriding
specific tables:

1. **collapse_tables** - Highest priority, forces specific tables to be collapsed regardless of other settings
2. **expand_tables** - Medium priority, forces specific tables to be expanded
3. **table_format** - Lowest priority, sets the default behavior for all tables not explicitly configured

This three-tier approach lets you fine-tune formatting for specific tables while maintaining a consistent default.
For example:

.. code-block:: toml

    [tool.pyproject-fmt]
    table_format = "short"  # Collapse most tables
    expand_tables = ["project.entry-points"]  # But expand entry-points

Specificity rules
~~~~~~~~~~~~~~~~~

Table selectors follow CSS-like specificity rules: more specific selectors win over less specific ones. When
determining whether to collapse or expand a table, the formatter checks from most specific to least specific until it
finds a match.

For example, with this configuration:

.. code-block:: toml

    [tool.pyproject-fmt]
    table_format = "long"  # Expand all tables by default
    collapse_tables = ["project"]  # Collapse project sub-tables
    expand_tables = ["project.optional-dependencies"]  # But expand this specific one

The behavior will be:

- ``project.urls`` → collapsed (matches ``project`` in collapse_tables)
- ``project.scripts`` → collapsed (matches ``project`` in collapse_tables)
- ``project.optional-dependencies`` → expanded (matches exactly in expand_tables, more specific than ``project``)
- ``tool.ruff.lint`` → expanded (no match in collapse/expand, uses table_format default)

This allows you to set broad rules for parent tables while making exceptions for specific sub-tables. The specificity
check walks up the table hierarchy: for ``project.optional-dependencies``, it first checks if
``project.optional-dependencies`` is in collapse_tables or expand_tables, then checks ``project``, then falls back to
the table_format default.

Supported tables
~~~~~~~~~~~~~~~~

The following sub-tables can be formatted with this configuration:

**Project tables:**

- ``project.urls`` - Project URLs (homepage, repository, documentation, changelog)
- ``project.scripts`` - Console script entry points
- ``project.gui-scripts`` - GUI script entry points
- ``project.entry-points`` - Custom entry point groups
- ``project.optional-dependencies`` - Optional dependency groups

**Tool tables:**

- ``tool.ruff.format`` - Ruff formatter settings
- ``tool.ruff.lint`` - Ruff linter settings
- Any other tool sub-tables

**Array of tables:**

- ``project.authors`` - Can be inline tables or ``[[project.authors]]``
- ``project.maintainers`` - Can be inline tables or ``[[project.maintainers]]``
- Any ``[[table]]`` entries throughout the file

Array of tables (``[[table]]``) are automatically collapsed to inline arrays when each inline table fits within the
configured ``column_width``. For example:

.. code-block:: toml

    # Before
    [[tool.commitizen.customize.questions]]
    type = "list"

    [[tool.commitizen.customize.questions]]
    type = "input"

    # After (with table_format = "short")
    [tool.commitizen]
    customize.questions = [{ type = "list" }, { type = "input" }]

If any inline table exceeds ``column_width``, the array of tables remains in ``[[...]]`` format to maintain
readability and TOML 1.0.0 compatibility (inline tables cannot span multiple lines).

String wrapping
---------------

By default, the formatter wraps long strings that exceed the column width using line continuations. However, some strings such as regex patterns should not be wrapped because wrapping can break their functionality.

You can configure which keys should skip string wrapping using the ``skip_wrap_for_keys`` option:

.. code-block:: toml

    [tool.pyproject-fmt]
    skip_wrap_for_keys = ["*.parse", "*.regex", "tool.bumpversion.*"]

Pattern matching
~~~~~~~~~~~~~~~~

The ``skip_wrap_for_keys`` option supports glob-like patterns:

- **Exact match**: ``tool.bumpversion.parse`` matches only that specific key
- **Wildcard suffix**: ``*.parse`` matches any key ending with ``.parse`` (e.g., ``tool.bumpversion.parse``, ``project.parse``)
- **Wildcard prefix**: ``tool.bumpversion.*`` matches any key under ``tool.bumpversion`` (e.g., ``tool.bumpversion.parse``, ``tool.bumpversion.serialize``)
- **Global wildcard**: ``*`` skips wrapping for all strings

Examples: ``["*.parse", "*.regex"]`` to preserve regex fields, ``["tool.bumpversion.*"]`` for a specific tool section,
or ``["*"]`` to skip all string wrapping.
