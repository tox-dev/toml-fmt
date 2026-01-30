tox-toml-fmt
=============

Apply a consistent format to your ``tox.toml`` file with comment support. See
`changelog here <https://github.com/tox-dev/toml-fmt/blob/main/tox-toml-fmt/CHANGELOG.md>`_.


Philosophy
----------
This tool aims to be an *opinionated formatter*, with similar objectives to `black <https://github.com/psf/black>`_.
This means it deliberately does not support a wide variety of configuration settings. In return, you get consistency,
predictability, and smaller diffs.

Formatting Principles
---------------------

``tox-toml-fmt`` applies the following formatting rules to your ``tox.toml`` file:

**Table Organization** - Tables are reordered into a consistent structure, ensuring that your tox configuration follows
a standard order. This makes it easier to navigate and understand the file.

**Comment Preservation** - All comments in your file are preserved during formatting, including inline comments and
comments before entries. Inline comments are aligned for better readability.

**Array and Dictionary Formatting** - Arrays and dictionaries are automatically expanded to multiple lines when they
exceed the configured column width. Trailing commas are added to multi-line arrays for consistency and to minimize diff
noise when adding new items. When within the column limit, they remain on a single line.

**Indentation and Column Width** - Your entire file is reformatted with consistent indentation and respects the
configured column width for line breaking decisions. The formatter maintains whitespace within inline tables and arrays
to preserve readability.

Normalizations and Transformations
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

In addition to the formatting principles above, ``tox-toml-fmt`` performs the following normalizations:

**Table Ordering** - The ``tox.toml`` file follows a standard table ordering where root level keys and
``[env_run_base]`` appear first, environment-specific sections (``[env.NAME]``) are ordered according to the
``env_list`` configuration if present, any additional environments not in ``env_list`` follow at the end, and this
respects your configuration and ensures reproducible ordering.

**Environment List Respect** - The formatter respects the ``env_list`` configuration option by ordering environments to
match the ``env_list`` order when specified, allowing you to control which test environments run first, and preserving
the ordering across formatting operations.

Use
---

Via ``CLI``
~~~~~~~~~~~

:pypi:`tox-toml-fmt` is a CLI tool that needs a Python interpreter (version 3.10 or higher) to run. We recommend
either :pypi:`pipx` or :pypi:`uv` to install tox-toml-fmt into an isolated environment. This has the added benefit that
later you will be able to upgrade tox-toml-fmt without affecting other parts of the system. We provide a method for
``pip`` too here, but we discourage that path if you can:

.. tab:: uv

    .. code-block:: bash

        # install uv per https://docs.astral.sh/uv/#getting-started
        uv tool install tox-toml-fmt
        tox-toml-fmt --help


.. tab:: pipx

    .. code-block:: bash

        python -m pip install pipx-in-pipx --user
        pipx install tox-toml-fmt
        tox-toml-fmt --help

.. tab:: pip

    .. code-block:: bash

        python -m pip install --user tox-toml-fmt
        tox-toml-fmt --help

    You can install it within the global Python interpreter itself (perhaps as a user package via the
    ``--user`` flag). Be cautious if you are using a Python installation that is managed by your operating system or
    another package manager. ``pip`` might not coordinate with those tools, and may leave your system in an inconsistent
    state. Note, if you go down this path you need to ensure pip is new enough per the subsections below


Via ``pre-commit`` hook
~~~~~~~~~~~~~~~~~~~~~~~

See :gh:`pre-commit/pre-commit` for instructions, sample ``.pre-commit-config.yaml``:

.. code-block:: yaml

    - repo: https://github.com/tox-dev/tox-toml-fmt
      rev: "v1.0.0"
      hooks:
        - id: tox-toml-fmt

Via Python
~~~~~~~~~~

You can use ``tox-toml-fmt`` as a Python module to format TOML content programmatically.

.. code-block:: python

    from tox_toml_fmt import run

    # Format a tox.toml file and return the exit code
    exit_code = run(["path/to/tox.toml"])

The ``run`` function accepts command-line arguments as a list and returns an exit code (0 for success, non-zero for
failure).

.. automodule:: tox_toml_fmt
   :members:

.. toctree::
   :hidden:

   self

Configuration via file
----------------------

The ``[tox-toml-fmt]`` table is used when present in the ``tox.toml`` file:

.. code-block:: toml

  [tox-toml-fmt]

  # After how many columns split arrays/dicts into multiple lines (1 forces always)
  column_width = 120

  # Number of spaces for indentation
  indent = 2

If not set they will default to values from the CLI. The example above shows the defaults.

Command line interface
----------------------
.. sphinx_argparse_cli::
  :module: tox_toml_fmt.__main__
  :func: _build_our_cli
  :prog: tox-toml-fmt
  :title:
