Configuration
=============

Project settings
----------------

Put per-file settings in ``[tox-toml-fmt]``:

.. code-block:: toml

    [tox-toml-fmt]
    column_width = 120
    indent = 2
    table_format = "short"
    sub_table_spacing = ""
    separate_root_table = "\n"
    expand_tables = []
    collapse_tables = []
    skip_wrap_for_keys = []
    pin_envs = []

These values match the command defaults. ``column_width`` controls array expansion and string wrapping. A trailing
comma keeps an array multiline regardless of its width. ``indent`` controls continuation indentation.

``table_format`` controls child tables below an environment. Environment tables retain their ``[env.NAME]`` headers.
``expand_tables`` and ``collapse_tables`` override the default by table path, and ``collapse_tables`` wins a tie.

``pin_envs`` writes named environments before the version-based order used for the rest of ``env_list`` and
``[env.NAME]`` tables.

Shared settings
---------------

A standalone ``tox-toml-fmt.toml`` can hold settings for several projects. The file uses the same keys without the
``[tox-toml-fmt]`` header:

.. code-block:: toml

    column_width = 120
    indent = 2
    table_format = "short"
    pin_envs = ["fix", "type"]

For each input, the formatter searches from the input's directory toward the filesystem root and uses the nearest
``tox-toml-fmt.toml``. ``--config`` selects a file directly:

.. code-block:: bash

    tox-toml-fmt --config /path/to/tox-toml-fmt.toml tox.toml

Command-line values establish defaults, the shared file overrides them, and ``[tox-toml-fmt]`` has final precedence.
The formatter validates file settings with the command-line converters. An unknown key or invalid value stops
formatting and reports its source.

Spacing
-------

``sub_table_spacing`` inserts text between child tables in one group. ``separate_root_table`` inserts text between root
groups. Each ``\n`` adds one blank line.

String wrapping
---------------

``skip_wrap_for_keys`` excludes matching keys from line-continuation wrapping:

.. code-block:: toml

    [tox-toml-fmt]
    skip_wrap_for_keys = ["*.commands", "env.*.description"]

``*`` matches one dotted key segment. A quoted ``"*"`` segment names a literal asterisk.

Command-line interface
----------------------

.. sphinx_argparse_cli::
    :module: tox_toml_fmt
    :func: build_parser
    :prog: tox-toml-fmt
    :title:
