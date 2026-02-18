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

Command line interface
----------------------

.. sphinx_argparse_cli::
    :module: tox_toml_fmt.__main__
    :func: _build_our_cli
    :prog: tox-toml-fmt
    :title:
