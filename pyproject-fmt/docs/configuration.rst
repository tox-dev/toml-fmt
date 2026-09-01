Configuration
=============

Project settings
----------------

Put per-file settings in ``[tool.pyproject-fmt]``:

.. code-block:: toml

    [tool.pyproject-fmt]
    column_width = 120
    indent = 2
    keep_full_version = false
    generate_python_version_classifiers = true
    max_supported_python = "3.14"
    table_format = "short"
    sub_table_spacing = ""
    separate_root_table = "\n"
    expand_tables = []
    collapse_tables = []
    skip_wrap_for_keys = []

These values match the command defaults. ``keep_full_version`` retains redundant zero components in dependency
versions. Classifier generation reads the lower bound from ``project.requires-python`` and uses
``max_supported_python`` as its upper bound.

``column_width`` controls array expansion and string wrapping. A trailing comma keeps an array multiline regardless of
its width. ``indent`` controls continuation indentation.

Shared settings
---------------

A standalone ``pyproject-fmt.toml`` can hold settings for several projects. The file uses the same keys without the
``[tool.pyproject-fmt]`` header:

.. code-block:: toml

    column_width = 120
    indent = 2
    table_format = "short"
    max_supported_python = "3.14"

For each input, the formatter searches from the input's directory toward the filesystem root and uses the nearest
``pyproject-fmt.toml``. ``--config`` selects a file directly:

.. code-block:: bash

    pyproject-fmt --config /path/to/pyproject-fmt.toml pyproject.toml

Command-line values establish defaults, the shared file overrides them, and ``[tool.pyproject-fmt]`` has final
precedence. The formatter validates file settings with the command-line converters. An unknown key or invalid value
stops formatting and reports its source.

Command-line interface
----------------------

.. sphinx_argparse_cli::
    :module: pyproject_fmt
    :func: build_parser
    :prog: pyproject-fmt
    :title:

Python classifiers
------------------

Classifier generation adds ``Programming Language :: Python :: 3.X`` entries for supported minor releases. The
``requires-python`` constraint supplies the lower edge and ``max_supported_python`` caps the result. The formatter
interprets the constraint under :pep:`440`:

- ``~=3.10`` includes 3.10 and later minor releases up to the configured cap.
- ``~=3.10.0`` includes the 3.10 series.
- ``!=3.10`` excludes that release, not the complete 3.10 series.
- A constraint with no matching Python 3 release, such as ``>=4``, produces no version classifiers.

Use ``generate_python_version_classifiers = false`` or ``--no-generate-python-version-classifiers`` to retain the
input classifier list.

Table shape
-----------

``table_format = "short"`` folds child tables into dotted keys. ``"long"`` writes child headers. Array-of-table
entries fold into inline tables when each entry fits within ``column_width``.

``expand_tables`` and ``collapse_tables`` override the default by table path. The closest matching path wins, with
``collapse_tables`` winning a tie. This configuration collapses most project children but expands optional
dependencies:

.. code-block:: toml

    [tool.pyproject-fmt]
    table_format = "long"
    collapse_tables = ["project"]
    expand_tables = ["project.optional-dependencies"]

The result folds ``project.urls`` and writes ``[project.optional-dependencies]`` as a header. Selectors also apply to
tool tables and arrays of tables.

Spacing
-------

``sub_table_spacing`` inserts text between child tables in one group. ``separate_root_table`` inserts text between root
groups. Each ``\n`` adds one blank line:

.. code-block:: toml

    [tool.pyproject-fmt]
    sub_table_spacing = "\n"
    separate_root_table = "\n"

String wrapping
---------------

``skip_wrap_for_keys`` excludes matching keys from line-continuation wrapping:

.. code-block:: toml

    [tool.pyproject-fmt]
    skip_wrap_for_keys = ["*.parse", "*.regex", "tool.bumpversion.*"]

Patterns match dotted key segments:

- ``tool.bumpversion.parse`` names one key.
- ``*.parse`` names any path ending in ``parse``.
- ``tool.bumpversion.*`` names direct children of ``tool.bumpversion``.
- ``tool.*.parse`` names one segment between ``tool`` and ``parse``.
- ``*`` names every key.

A quoted ``"*"`` segment names a literal asterisk.
