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

    # Table format: "short" collapses sub-tables to dotted keys, "long" expands to
    # [table.subtable] headers
    table_format = "short"

    # Extra newlines between sub-tables in the same group (e.g. "\n" for one blank line
    # between sub-tables)
    sub_table_spacing = ""

    # Extra newlines between root table groups (e.g. "\n" for one blank line, "\n\n" for two)
    separate_root_table = "\n"

    # List of tables to force expand regardless of table_format setting
    expand_tables = []

    # List of tables to force collapse regardless of table_format or expand_tables settings
    collapse_tables = []

    # List of key patterns to skip string wrapping (supports wildcards like *.parse or
    # tool.bumpversion.*)
    skip_wrap_for_keys = []

If not set they will default to values from the CLI.

Shared configuration file
-------------------------

Place formatting settings in a standalone ``pyproject-fmt.toml`` file instead of (or alongside) the
``[tool.pyproject-fmt]`` table. In a monorepo this shares one configuration across projects without repeating it in
every ``pyproject.toml``.

The formatter searches for ``pyproject-fmt.toml`` from the directory of the file being formatted up to the filesystem
root, and the first match wins. Pass an explicit path via ``--config``:

.. code-block:: bash

    pyproject-fmt --config /path/to/pyproject-fmt.toml pyproject.toml

The shared config file uses the same keys as the ``[tool.pyproject-fmt]`` table, but without the table header:

.. code-block:: toml

    column_width = 120
    indent = 2
    table_format = "short"
    sub_table_spacing = ""
    separate_root_table = "\n"
    max_supported_python = "3.14"

When both a shared config file and a ``[tool.pyproject-fmt]`` table exist, per-file settings from the
``[tool.pyproject-fmt]`` table take precedence over the shared config file.

Settings are read with the same parser that reads the file, so a value only TOML 1.1 spells does not
hide the table they are written in. Every key there has to be one the formatter knows, written as the
type its command-line flag takes; anything else is reported against the file and the key, and nothing
is formatted.

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

Within that window a minor version gets its classifier when some release of that series satisfies every clause of
``requires-python``, the way :pep:`440` reads one. ``~=3.10`` therefore covers 3.10 and everything after it up to the
upper bound, ``~=3.10.0`` covers only the 3.10 series, ``!=3.10`` rules out that one release rather than the series,
and a constraint no Python 3 release satisfies, such as ``>=4``, generates no classifiers at all.

Table formatting
----------------

.. note::

    Table formatting options are available in version 2.12.0 and later.

``table_format`` picks between the two styles: ``short``, the default, collapses a sub-table into dotted keys, and
``long`` writes it out under its own ``[table.subtable]`` header. The formatting guide shows what each one produces.

Table spacing
~~~~~~~~~~~~~

The ``sub_table_spacing`` and ``separate_root_table`` options control the blank lines inserted between tables. Each
option takes a string of ``\n`` characters where each ``\n`` adds one blank line:

- ``sub_table_spacing`` (default ``""``) controls spacing between sub-tables within the same group. For example,
  between ``[tool.ruff]`` and ``[tool.ruff.lint]``. Set to ``"\n"`` to add a blank line between sub-tables.
- ``separate_root_table`` (default ``"\n"``) controls spacing between different root table groups. For example,
  between ``[project]`` and ``[tool.ruff]``.

.. code-block:: toml

    [tool.pyproject-fmt]
    sub_table_spacing = "\n"  # Add blank line between sub-tables
    separate_root_table = "\n"  # One blank line between root table groups (default)

Configuration priority
~~~~~~~~~~~~~~~~~~~~~~

A priority system sets a global default while letting you override specific tables:

1. **collapse_tables** - Highest priority, forces specific tables to collapse regardless of other settings
2. **expand_tables** - Medium priority, forces specific tables to expand
3. **table_format** - Lowest priority, sets the default for all tables not configured above

Set a broad default, then carve out exceptions per table. For example:

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

An array of tables collapses into inline tables where each one fits the configured ``column_width``; the formatting
guide shows what that looks like and when it stays written out.

String wrapping
---------------

By default the formatter wraps strings past the column width using line continuations. Some strings, regex patterns
especially, break when wrapped, so exclude their keys with ``skip_wrap_for_keys``:

.. code-block:: toml

    [tool.pyproject-fmt]
    skip_wrap_for_keys = ["*.parse", "*.regex", "tool.bumpversion.*"]

Pattern matching
~~~~~~~~~~~~~~~~

The ``skip_wrap_for_keys`` option supports glob-like patterns:

- **Exact match**: ``tool.bumpversion.parse`` matches only that specific key
- **Wildcard suffix**: ``*.parse`` matches any key ending with ``.parse`` (e.g., ``tool.bumpversion.parse``, ``project.parse``)
- **Wildcard prefix**: ``tool.bumpversion.*`` matches any key under ``tool.bumpversion`` (e.g., ``tool.bumpversion.parse``, ``tool.bumpversion.serialize``)
- **Wildcard between names**: ``tool.*.parse`` stands for one segment, so it matches ``tool.bumpversion.parse`` but not
  a key written below it
- **Global wildcard**: ``*`` skips wrapping for all strings

A quoted ``"*"`` names the key spelled that way rather than standing for any segment.

Examples: ``["*.parse", "*.regex"]`` to preserve regex fields, ``["tool.bumpversion.*"]`` for a specific tool section,
or ``["*"]`` to skip all string wrapping.
