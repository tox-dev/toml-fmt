pyproject-fmt
=============

Apply a consistent format to your ``pyproject.toml`` file with comment support. See
`changelog here <https://github.com/tox-dev/toml-fmt/blob/main/pyproject-fmt/CHANGELOG.md>`_.


Philosophy
----------
This tool aims to be an *opinionated formatter*, with similar objectives to `black <https://github.com/psf/black>`_.
This means it deliberately does not support a wide variety of configuration settings. In return, you get consistency,
predictability, and smaller diffs.

Formatting Principles
---------------------

``pyproject-fmt`` is an opinionated formatter, much like `black <https://github.com/psf/black>`_ is for Python code.
The tool intentionally provides minimal configuration options because the goal is to establish a single standard format
that all ``pyproject.toml`` files follow. Rather than spending time debating formatting preferences, teams can simply
run ``pyproject-fmt`` and have consistent, predictable results. This opinionated approach has benefits: less time
configuring tools, smaller diffs when committing changes, and easier code reviews since formatting is never a question.
While a few key options exist (``column_width``, ``indent``, ``table_format``, etc.), the tool does not expose dozens of
toggles. You get what the maintainers have chosen to be the right balance of readability, consistency, and usability.

``pyproject-fmt`` applies the following formatting rules to your ``pyproject.toml`` file:

**Table Organization** - Tables are reordered into a consistent structure. The ``[build-system]`` section appears
first, followed by ``[project]``, then ``[tool]`` sections in a defined order (e.g., ``tool.ruff`` before other tools),
and finally other tables.

**Comment Preservation** - All comments in your file are preserved during formatting, including inline comments and
comments before entries. Comments are aligned for better readability.

**Array and Dictionary Formatting** - Arrays and dictionaries are automatically expanded to multiple lines when they
exceed the configured column width. Trailing commas are added to multi-line arrays for consistency. When within the
column limit, they remain on a single line.

**Indentation and Column Width** - Your entire file is reformatted with consistent indentation and respects the
configured column width for line breaking decisions.

**Table Formatting** - Sub-tables can be formatted as either collapsed dotted keys (``project.urls.homepage``) or
expanded table headers (``[project.urls]``). You can configure global defaults and override specific tables.

Normalizations and Transformations
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

In addition to the formatting principles above, ``pyproject-fmt`` performs the following normalizations:

**Version Specifiers** - All PEP 508 version specifiers are normalized by removing spaces around operators
(e.g., ``package >= 1.0`` becomes ``package>=1.0``), optionally removing redundant trailing zeros (e.g., ``1.0``
becomes ``1``) unless ``keep_full_version = true``, and validating against PEP 508 standards.

**Dependency Sorting** - Dependencies are sorted by their canonical package name. In ``[build-system]`` ``requires``
arrays, dependencies are sorted alphabetically. In ``[dependency-groups]``, dependencies are sorted with regular
requirements first, then include-group entries.

**Python Version Classifiers** - Programming Language classifiers for Python versions are automatically generated
based on the ``requires-python`` field (which sets the lower bound, defaults to oldest supported Python), the
``max_supported_python`` configuration option (which sets the upper bound), and can be disabled with
``generate_python_version_classifiers = false``.

**Authors and Maintainers** - Contact information can be collapsed to inline tables on the ``project.authors`` or
``project.maintainers`` line, or expanded to full ``[[project.authors]]`` array of tables format, and this is
controlled by ``table_format``, ``expand_tables``, and ``collapse_tables`` configuration options.

**Ruff Configuration** - The ``[tool.ruff]`` section receives special formatting where sub-tables (``tool.ruff.lint``,
``tool.ruff.format``, etc.) can be collapsed or expanded based on configuration, and lists like ``exclude``,
``include``, and rule selections are sorted.

**Entry Points** - The ``[project.entry-points]`` section is formatted as inline tables for compactness unless
expanded by configuration.

Handled Tables
~~~~~~~~~~~~~~

``pyproject-fmt`` applies intelligent formatting to the following tables and sections in your ``pyproject.toml`` file.

The ``[build-system]`` table receives special attention: dependencies in the ``requires`` array are normalized and
sorted alphabetically by canonical package name. The table keys are reordered to a consistent order: ``build-backend``,
``requires``, then ``backend-path``.

The ``[project]`` table is formatted with reorered keys in a logical order and receives transformations on several
sub-sections. Authors and maintainers can be formatted as inline tables or expanded array of tables. Project URLs,
scripts, GUI scripts, and entry points are formatted according to your ``table_format`` configuration and can be
individually controlled via ``expand_tables`` and ``collapse_tables``. Python version classifiers are automatically
maintained based on your ``requires-python`` setting.

The ``[dependency-groups]`` table (introduced in PEP 735) is processed where all dependencies are normalized according
to PEP 508 and sorted. Each dependency group is treated as an array, and entries can be simple strings or inline tables
with include-group references, which are sorted appropriately.

Tool-specific sections under ``[tool]`` follow a predefined order to ensure consistency across projects. The formatter
recognizes over 50 tool sections (including poetry, pdm, setuptools, ruff, mypy, pytest, tox, coverage, and many others)
and orders them in a standard sequence. The ``[tool.ruff]`` section receives special treatment where sub-sections like
``lint``, ``format``, and others can be collapsed or expanded, and configuration lists are sorted.

Any other tables in your file are preserved and reordered according to the standard table ordering rules, but receive
minimal transformation unless they match one of the recognized patterns above.

Use
---

Via ``CLI``
~~~~~~~~~~~

:pypi:`pyproject-fmt` is a CLI tool that needs a Python interpreter (version 3.10 or higher) to run. We recommend
either :pypi:`pipx` or :pypi:`uv` to install pyproject-fmt into an isolated environment. This has the added benefit that
later you will be able to upgrade pyproject-fmt without affecting other parts of the system. We provide a method for
``pip`` too here, but we discourage that path if you can:

.. tab:: uv

    .. code-block:: bash

        # install uv per https://docs.astral.sh/uv/#getting-started
        uv tool install pyproject-fmt
        pyproject-fmt --help


.. tab:: pipx

    .. code-block:: bash

        python -m pip install pipx-in-pipx --user
        pipx install pyproject-fmt
        pyproject-fmt --help

.. tab:: pip

    .. code-block:: bash

        python -m pip install --user pyproject-fmt
        pyproject-fmt --help

    You can install it within the global Python interpreter itself (perhaps as a user package via the
    ``--user`` flag). Be cautious if you are using a Python installation that is managed by your operating system or
    another package manager. ``pip`` might not coordinate with those tools, and may leave your system in an inconsistent
    state. Note, if you go down this path you need to ensure pip is new enough per the subsections below


Via ``pre-commit`` hook
~~~~~~~~~~~~~~~~~~~~~~~

See :gh:`pre-commit/pre-commit` for instructions, sample ``.pre-commit-config.yaml``:

.. code-block:: yaml

    - repo: https://github.com/tox-dev/pyproject-fmt
      # Use the sha / tag you want to point at
      # or use `pre-commit autoupdate` to get the latest version
      rev: ""
      hooks:
        - id: pyproject-fmt

Via Python
~~~~~~~~~~

You can use ``pyproject-fmt`` as a Python module to format TOML content programmatically.

.. code-block:: python

    from pyproject_fmt import run

    # Format a pyproject.toml file and return the exit code
    exit_code = run(["path/to/pyproject.toml"])

The ``run`` function accepts command-line arguments as a list and returns an exit code (0 for success, non-zero for
failure).

.. automodule:: pyproject_fmt
   :members:

.. toctree::
   :hidden:

   self

Configuration via file
----------------------

The ``tool.pyproject-fmt`` table is used when present in the ``pyproject.toml`` file:

.. code-block:: toml

  [tool.pyproject-fmt]

  # After how many columns split arrays/dicts into multiple lines (1 forces always)
  column_width = 120

  # Number of spaces for indentation
  indent = 2

  # Keep full version numbers (e.g., 1.0.0 instead of 1.0) in dependency specifiers
  keep_full_version = false

  # Automatically generate Python version classifiers based on requires-python
  # Set to false to disable automatic classifier generation
  generate_python_version_classifiers = true

  # Maximum Python version for generating version classifiers
  max_supported_python = "3.12"

  # Table format: "short" collapses sub-tables to dotted keys, "long" expands to [table.subtable] headers
  table_format = "short"

  # List of tables to force expand regardless of table_format setting
  expand_tables = ["project.entry-points", "project.optional-dependencies"]

  # List of tables to force collapse regardless of table_format or expand_tables settings
  collapse_tables = ["project.urls"]

If not set they will default to values from the CLI. The example above shows the defaults.

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
  the oldest non end of line CPython version at the time of the release).
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

Supported tables
~~~~~~~~~~~~~~~~

The following sub-tables can be formatted with this configuration:

**Project tables:**

- ``project.urls`` - Project URLs (homepage, repository, etc.)
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
