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

    # Environments pinned to the start of env_list
    pin_envs = ["fix", "type"]

If not set they will default to values from the CLI. The example above shows the defaults (except ``pin_envs``
which defaults to an empty list).

Shared configuration file
-------------------------

You can place formatting settings in a standalone ``tox-toml-fmt.toml`` file instead of (or in addition to) the
``[tox-toml-fmt]`` table. This is useful for monorepos or when you want to share the same configuration across multiple
projects without duplicating it in each ``tox.toml``.

The formatter searches for ``tox-toml-fmt.toml`` starting from the directory of the file being formatted and walking up
to the filesystem root. The first match wins. You can also pass an explicit path via ``--config``:

.. code-block:: bash

    tox-toml-fmt --config /path/to/tox-toml-fmt.toml tox.toml

The shared config file uses the same keys as the ``[tox-toml-fmt]`` table, but without the table header:

.. code-block:: toml

    column_width = 120
    indent = 2
    pin_envs = ["fix", "type"]

When both a shared config file and a ``[tox-toml-fmt]`` table exist, per-file settings from the ``[tox-toml-fmt]``
table take precedence over the shared config file.

Command line interface
----------------------

.. sphinx_argparse_cli::
    :module: tox_toml_fmt.__main__
    :func: _build_our_cli
    :prog: tox-toml-fmt
    :title:
