Configuration
=============

Configuration via file
----------------------

The ``[tox-toml-fmt]`` table is used when present in the ``tox.toml`` file:

.. code-block:: toml

    [tox-toml-fmt]

    # After how many columns split arrays/dicts into multiple lines and wrap long strings;
    # use a trailing comma in arrays to force multiline format instead of lowering this value
    column_width = 120

    # Number of spaces for indentation
    indent = 2

    # Extra newlines between sub-tables in the same group (e.g. "\n" for one blank line
    # between sub-tables)
    sub_table_spacing = ""

    # Extra newlines between root table groups (e.g. "\n" for one blank line, "\n\n" for two)
    separate_root_table = "\n"

    # Environments pinned to the start of env_list
    pin_envs = ["fix", "type"]

If not set they will default to values from the CLI. The example above shows the defaults (except ``pin_envs``
which defaults to an empty list).

Shared configuration file
-------------------------

Place formatting settings in a standalone ``tox-toml-fmt.toml`` file instead of (or alongside) the ``[tox-toml-fmt]``
table. In a monorepo this shares one configuration across projects without repeating it in every ``tox.toml``.

The formatter searches for ``tox-toml-fmt.toml`` from the directory of the file being formatted up to the filesystem
root, and the first match wins. Pass an explicit path via ``--config``:

.. code-block:: bash

    tox-toml-fmt --config /path/to/tox-toml-fmt.toml tox.toml

The shared config file uses the same keys as the ``[tox-toml-fmt]`` table, but without the table header:

.. code-block:: toml

    column_width = 120
    indent = 2
    sub_table_spacing = ""
    separate_root_table = "\n"
    pin_envs = ["fix", "type"]

When both a shared config file and a ``[tox-toml-fmt]`` table exist, per-file settings from the ``[tox-toml-fmt]``
table take precedence over the shared config file.

Settings are read with the same parser that reads the file, so a value only TOML 1.1 spells does not hide the table
they are written in. Every key there has to be one the formatter knows, written as the type its command-line flag
takes; anything else is reported against the file and the key, and nothing is formatted.

Command line interface
----------------------

.. sphinx_argparse_cli::
    :module: tox_toml_fmt
    :func: build_parser
    :prog: tox-toml-fmt
    :title:
