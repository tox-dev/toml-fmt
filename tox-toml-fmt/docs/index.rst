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

``tox-toml-fmt`` is an opinionated formatter, much like `black <https://github.com/psf/black>`_ is for Python code.
The tool intentionally provides minimal configuration options because the goal is to establish a single standard format
that all ``tox.toml`` files follow. Rather than spending time debating formatting preferences, teams can simply run
``tox-toml-fmt`` and have consistent, predictable results. This opinionated approach has benefits: less time configuring
tools, smaller diffs when committing changes, and easier code reviews since formatting is never a question. While a few
key options exist (``column_width``, ``indent``), the tool does not expose dozens of toggles. You get what the
maintainers have chosen to be the right balance of readability, consistency, and usability.

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

Handled Tables
~~~~~~~~~~~~~~

``tox-toml-fmt`` applies formatting to the following sections in your ``tox.toml`` file.

The root-level configuration including keys like ``env_list``, ``min_version``, ``skip_missing_interpreters``, and
other global tox settings appear at the top of your formatted file and are preserved in their original form.

The ``[env_run_base]`` section, which defines base configuration inherited by all environments, is positioned after
root-level settings. This is a special section that sets up shared behavior for test environments, like shared
dependencies or common settings that apply across the board.

Environment-specific sections follow the pattern ``[env.NAME]`` where ``NAME`` is the environment identifier (such as
``py38``, ``py39``, ``lint``, ``type-check``, or any custom environment you define). These sections are ordered
according to your ``env_list`` configuration if specified, ensuring that environments run in your preferred sequence.
Any environments not explicitly listed in ``env_list`` are placed at the end, maintaining deterministic ordering across
formatting operations.

Within environment sections, configuration keys are preserved and arrays/dictionaries are formatted according to column
width and indentation settings, just like in pyproject.toml. Comments are fully preserved and aligned for readability.

Any other custom sections or keys in your tox.toml file are preserved and positioned according to the standard table
ordering, ensuring your configuration remains valid and your custom settings are maintained.

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
