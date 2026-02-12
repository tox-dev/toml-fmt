Formatting Rules
================

``tox-toml-fmt`` is an opinionated formatter, much like `black <https://github.com/psf/black>`_ is for Python code.
The tool intentionally provides minimal configuration options because the goal is to establish a single standard format
that all ``tox.toml`` files follow.

**Benefits of this approach:**

- Less time configuring tools
- Smaller diffs when committing changes
- Easier code reviews since formatting is never a question

While a few key options exist (``column_width``, ``indent``, ``table_format``), the tool does not expose dozens of
toggles. You get what the maintainers have chosen to be the right balance of readability, consistency, and usability.

General Formatting
------------------

These rules apply uniformly across the entire ``tox.toml`` file.

String Quotes
~~~~~~~~~~~~~

All strings use double quotes by default. Single quotes are only used when the value contains double quotes:

.. code-block:: toml

    # Before
    description = 'Run tests'
    commands = ["echo \"hello\""]

    # After
    description = "Run tests"
    commands = ['echo "hello"']

Key Quotes
~~~~~~~~~~

TOML keys using single-quoted (literal) strings are normalized to double-quoted (basic) strings with proper escaping.
This ensures consistent formatting and deterministic key sorting regardless of the original quote style:

.. code-block:: toml

    # Before
    [env.'my-env']
    deps = ["pytest"]

    # After
    [env."my-env"]
    deps = ["pytest"]

Backslashes and double quotes within literal keys are escaped during conversion.

Array Formatting
~~~~~~~~~~~~~~~~

Arrays are formatted based on line length, trailing comma presence, and comments:

.. code-block:: toml

    # Short arrays stay on one line
    env_list = ["py312", "py313", "lint"]

    # Long arrays that exceed column_width are expanded and get a trailing comma
    deps = [
        "pytest>=7",
        "pytest-cov>=4",
        "pytest-mock>=3",
    ]

    # Trailing commas signal intent to keep multiline format
    deps = [
        "pytest>=7",
    ]

    # Arrays with comments are always multiline
    deps = [
        "pytest>=7",  # testing framework
        "coverage>=7",
    ]

**Multiline formatting rules:**

An array becomes multiline when any of these conditions are met:

1. **Trailing comma present** - A trailing comma signals intent to keep multiline format
2. **Exceeds column width** - Arrays longer than ``column_width`` are expanded (and get a trailing comma added)
3. **Contains comments** - Arrays with inline or leading comments are always multiline

String Wrapping
~~~~~~~~~~~~~~~

Long strings that exceed ``column_width`` are wrapped using TOML multiline basic strings with line-ending backslashes:

.. code-block:: toml

    # Before
    description = "A very long description string that exceeds the column width limit set for this project"

    # After (with column_width = 40)
    description = """\
      A very long description \
      string that exceeds the \
      column width limit set \
      for this project\
      """

Specific keys can be excluded from wrapping using ``skip_wrap_for_keys``. Patterns support wildcards
(e.g. ``*.commands`` skips wrapping for ``commands`` under any table).

Table Formatting
~~~~~~~~~~~~~~~~

Sub-tables can be formatted in two styles controlled by ``table_format``:

**Short format** (default, collapsed to dotted keys):

.. code-block:: toml

    [env.test]
    description = "run tests"
    sub.value = 1

**Long format** (expanded to table headers):

.. code-block:: toml

    [env.test]
    description = "run tests"

    [env.test.sub]
    value = 1

Individual tables can override the default using ``expand_tables`` and ``collapse_tables``.
See :doc:`configuration` for how to control this behavior.

Comment Preservation
~~~~~~~~~~~~~~~~~~~~

All comments are preserved during formatting:

- **Inline comments** - Comments after a value on the same line stay with that value
- **Leading comments** - Comments on the line before an entry stay with the entry below
- **Block comments** - Multi-line comment blocks are preserved

**Inline comment alignment:**

Inline comments within arrays are aligned independently per array, based on that array's longest value:

.. code-block:: toml

    # Before - comments at inconsistent positions
    deps = [
      "pytest", # testing
      "pytest-cov",  # coverage
      "pytest-mock", # mocking
    ]

    # After - comments align to longest value in this array
    deps = [
      "pytest",       # testing
      "pytest-cov",   # coverage
      "pytest-mock",  # mocking
    ]

Table-Specific Handling
-----------------------

Table Ordering
~~~~~~~~~~~~~~

Tables are reordered into a consistent structure:

1. Root-level keys (``env_list``, ``min_version``, ``skip_missing_interpreters``)
2. ``[env_run_base]``
3. ``[env.NAME]`` sections ordered by ``env_list`` if specified
4. Any remaining ``[env.*]`` sections not in ``env_list``

.. code-block:: toml

    # env_list determines the order of [env.*] sections
    env_list = ["lint", "type", "py312", "py313"]

    [env_run_base]
    deps = ["pytest>=7"]

    # Environments appear in env_list order:
    [env.lint]
    # ...

    [env.type]
    # ...

    [env.py312]
    # ...

    [env.py313]
    # ...

Environments not listed in ``env_list`` are placed at the end.

Other Tables
~~~~~~~~~~~~

Any unrecognized tables are preserved and reordered according to standard table ordering rules. Keys within tables
are not reordered.
