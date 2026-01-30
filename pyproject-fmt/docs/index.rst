pyproject-fmt
=============

Apply a consistent format to your ``pyproject.toml`` file with comment support. See
`changelog here <https://github.com/tox-dev/toml-fmt/blob/main/pyproject-fmt/CHANGELOG.md>`_.


Philosophy
----------
This tool aims to be an *opinionated formatter*, with similar objectives to `black <https://github.com/psf/black>`_.
This means it deliberately does not support a wide variety of configuration settings. In return, you get consistency,
predictability, and smaller diffs.

Use
---

Via ``CLI``
~~~~~~~~~~~

:pypi:`pyproject-fmt` is a CLI tool that needs a Python interpreter (version 3.10 or higher) to run. We recommend either
:pypi:`pipx` or :pypi:`uv` to install pyproject-fmt into an isolated environment. This has the added benefit that later you'll
be able to upgrade pyproject-fmt without affecting other parts of the system. We provide method for ``pip`` too here but we
discourage that path if you can:

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

  # after how many column width split arrays/dicts into multiple lines, 1 will force always
  column_width = 120

  # how many spaces use for indentation
  indent = 2

  # if false will remove unnecessary trailing ``.0``'s from version specifiers
  keep_full_version = false

  # maximum Python version to use when generating version specifiers
  max_supported_python = "3.12"

  # table format: "short" collapses sub-tables to dotted keys, "long" expands to [table.subtable] headers
  table_format = "short"

  # list of tables to force expand regardless of table_format setting
  expand_tables = ["project.entry-points", "project.optional-dependencies"]

  # list of tables to force collapse regardless of table_format or expand_tables settings
  collapse_tables = ["project.urls"]

If not set they will default to values from the CLI, the example above shows the defaults.

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

You can control how sub-tables are formatted in your ``pyproject.toml`` file. There are two formatting styles:

**Short format (collapsed)** - The default behavior where sub-tables are collapsed into dotted keys:

.. code-block:: toml

  [project]
  name = "myproject"
  urls.homepage = "https://example.com"
  urls.repository = "https://github.com/example/myproject"
  scripts.mycli = "mypackage:main"

**Long format (expanded)** - Sub-tables are expanded into separate ``[table.subtable]`` sections:

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

The formatting behavior is determined by a priority system:

1. **collapse_tables** - Highest priority, forces specific tables to be collapsed
2. **expand_tables** - Medium priority, forces specific tables to be expanded
3. **table_format** - Lowest priority, sets the default behavior for all tables

This allows you to set a global default and override specific tables. For example:

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
