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

TOML keys are normalized to the simplest valid form. Keys that are valid bare keys (containing only
``A-Za-z0-9_-``) have redundant quotes stripped. Single-quoted (literal) keys that require quoting are
converted to double-quoted (basic) strings with proper escaping. This applies to all keys: table headers,
key-value pairs, and inline table keys:

.. code-block:: toml

    # Before
    [env.'my-env']
    "description" = "run tests"
    pass_env = [{ "else" = "no" }]

    # After
    [env."my-env"]
    description = "run tests"
    pass_env = [{ else = "no" }]

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

**Environment tables are always expanded:**

Regardless of the ``table_format`` setting, ``[env.*]`` tables are never collapsed into dotted keys under ``[env]``.
Each environment always gets its own ``[env.NAME]`` table section:

.. code-block:: toml

    # This is always the output format, even in short mode:
    [env.fix]
    description = "fix"

    [env.test]
    description = "test"

    # Dotted keys under [env] are automatically expanded:
    # [env]
    # fix.description = "fix"    →    [env.fix]
    #                                  description = "fix"

Sub-tables within an environment (e.g. ``[env.test.sub]``) still follow the ``table_format`` setting.

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

Beyond general formatting, tables have specific key ordering, value normalization, and sorting rules.

Table Ordering
~~~~~~~~~~~~~~

Tables are reordered into a consistent structure:

1. Root-level keys (``min_version``, ``requires``, ``env_list``, etc.)
2. ``[env_run_base]``
3. ``[env_pkg_base]``
4. ``[env_base.*]`` sections (shared base configurations)
5. ``[env.NAME]`` sections ordered by ``env_list`` if specified
6. Any remaining ``[env.*]`` sections not in ``env_list``, sorted alphabetically
7. ``[env]`` (catch-all environment table, if present)

.. code-block:: toml

    # env_list determines the order of [env.*] sections
    env_list = ["lint", "type", "py312", "py313"]

    [env_run_base]
    deps = ["pytest>=7"]

    [env_pkg_base]
    # ...

    [env_base.ci]
    # shared base config

    # Environments appear in env_list order:
    [env.lint]
    # ...

    [env.type]
    # ...

    [env.py312]
    # ...

    [env.py313]
    # ...

Environments not listed in ``env_list`` are placed at the end, sorted alphabetically.

Alias Normalization
~~~~~~~~~~~~~~~~~~~

Legacy INI-style key names are renamed to their modern tox 4 TOML equivalents. This applies automatically
to the root table, ``[env_run_base]``, ``[env_pkg_base]``, and all ``[env.*]`` tables.

**Root table aliases:**

.. code-block:: toml

    # Before
    envlist = ["py312", "py313"]
    minversion = "4.2"
    skipsdist = true

    # After
    env_list = ["py312", "py313"]
    min_version = "4.2"
    no_package = true

Full list: ``envlist`` → ``env_list``, ``toxinidir`` → ``tox_root``, ``toxworkdir`` → ``work_dir``,
``skipsdist`` → ``no_package``, ``isolated_build_env`` → ``package_env``, ``setupdir`` → ``package_root``,
``minversion`` → ``min_version``, ``ignore_basepython_conflict`` → ``ignore_base_python_conflict``

**Environment table aliases:**

.. code-block:: toml

    # Before
    [env_run_base]
    basepython = "python3.12"
    setenv.PYTHONPATH = "src"
    passenv = ["HOME"]

    # After
    [env_run_base]
    base_python = "python3.12"
    set_env.PYTHONPATH = "src"
    pass_env = ["HOME"]

Full list: ``setenv`` → ``set_env``, ``passenv`` → ``pass_env``, ``envdir`` → ``env_dir``,
``envtmpdir`` → ``env_tmp_dir``, ``envlogdir`` → ``env_log_dir``, ``changedir`` → ``change_dir``,
``basepython`` → ``base_python``, ``usedevelop`` → ``use_develop``, ``sitepackages`` →
``system_site_packages``, ``alwayscopy`` → ``always_copy``

Root Key Ordering
~~~~~~~~~~~~~~~~~

Keys in the root table are reordered into a consistent sequence:

``min_version`` → ``requires`` → ``provision_tox_env`` → ``env_list`` → ``labels`` → ``base`` →
``package_env`` → ``package_root`` → ``no_package`` → ``skip_missing_interpreters`` →
``ignore_base_python_conflict`` → ``work_dir`` → ``temp_dir`` → ``tox_root``

.. code-block:: toml

    # Before
    env_list = ["py312", "lint"]
    requires = ["tox>=4.2"]
    min_version = "4.2"

    # After
    min_version = "4.2"
    requires = ["tox>=4.2"]
    env_list = ["py312", "lint"]

Environment Key Ordering
~~~~~~~~~~~~~~~~~~~~~~~~~

Keys within ``[env_run_base]``, ``[env_pkg_base]``, and ``[env.*]`` tables are reordered to group related
settings:

``factors`` → ``runner`` → ``description`` → ``base_python`` → ``default_base_python`` →
``system_site_packages`` → ``always_copy`` → ``download`` → ``virtualenv_spec`` → ``package`` →
``package_env`` → ``wheel_build_env`` → ``package_tox_env_type`` → ``package_root`` →
``skip_install`` → ``use_develop`` → ``meta_dir`` → ``pkg_dir`` → ``pip_pre`` →
``install_command`` → ``list_dependencies_command`` → ``deps`` → ``dependency_groups`` →
``pylock`` → ``constraints`` → ``constrain_package_deps`` → ``use_frozen_constraints`` → ``extras`` →
``recreate`` → ``recreate_commands`` → ``parallel_show_output`` → ``skip_missing_interpreters`` →
``fail_fast`` → ``pass_env`` → ``disallow_pass_env`` → ``set_env`` → ``change_dir`` →
``platform`` → ``args_are_paths`` → ``ignore_errors`` → ``commands_retry`` → ``ignore_outcome`` →
``extra_setup_commands`` → ``commands_pre`` → ``commands`` → ``commands_post`` →
``allowlist_externals`` → ``labels`` → ``suicide_timeout`` → ``interrupt_timeout`` →
``terminate_timeout`` → ``depends`` → ``env_dir`` → ``env_tmp_dir`` → ``env_log_dir``

.. code-block:: toml

    # Before
    [env_run_base]
    commands = ["pytest"]
    deps = ["pytest>=7"]
    description = "run tests"

    # After
    [env_run_base]
    description = "run tests"
    deps = ["pytest>=7"]
    commands = ["pytest"]

``requires`` Normalization
~~~~~~~~~~~~~~~~~~~~~~~~~~

Dependencies in the root ``requires`` array are normalized per PEP 508 (canonical package names,
consistent spacing around specifiers) and sorted alphabetically by package name:

.. code-block:: toml

    # Before
    requires = ["tox >= 4.2", "tox-uv"]

    # After
    requires = ["tox>=4.2", "tox-uv"]

``env_list`` Sorting
~~~~~~~~~~~~~~~~~~~~

The ``env_list`` array is sorted with a specific ordering:

1. **Pinned environments** come first, in the order specified by ``--pin-env``
2. **CPython versions** (matching ``py3.12``, ``py312``, ``3.12``, etc.) sorted descending (newest first)
3. **PyPy versions** (matching ``pypy3.10``, ``pypy310``, etc.) sorted descending
4. **Named environments** (``lint``, ``type``, ``docs``, etc.) sorted alphabetically

Inline table entries (such as ``{ product = ... }``) in ``env_list`` are excluded from sorting and remain
in their original positions.

Compound environment names separated by ``-`` are classified by their first recognized part:

.. code-block:: toml

    # Before
    env_list = ["lint", "py38", "py312", "docs", "py310-django"]

    # After
    env_list = ["py312", "py310-django", "py38", "docs", "lint"]

Use ``--pin-env`` to pin specific environments to the start:

.. code-block:: toml

    # With --pin-env fix,type
    env_list = ["fix", "type", "py313", "py312", "docs", "lint"]

See :doc:`configuration` for how to set ``pin-env`` via the config file or CLI.

``use_develop`` Upgrade
~~~~~~~~~~~~~~~~~~~~~~~

The legacy ``use_develop = true`` setting is automatically converted to the modern ``package = "editable"``
equivalent. If ``use_develop = false``, the key is left as-is. If a ``package`` key already exists,
only the ``use_develop`` key is removed:

.. code-block:: toml

    # Before
    [env_run_base]
    use_develop = true

    # After
    [env_run_base]
    package = "editable"

Array Sorting
~~~~~~~~~~~~~

Certain arrays within environment tables are sorted automatically:

**Sorted by canonical PEP 508 package name:**

- ``deps``, ``constraints`` — dependencies normalized and sorted by package name

Pip file references (``-r requirements.txt``, ``-c constraints.txt``) are preserved as-is without
PEP 508 normalization, but still participate in sorting by their lowercased value:

.. code-block:: toml

    # Before
    deps = ["Pytest >= 7", "-r requirements.txt", "coverage", "pytest-mock"]

    # After
    deps = ["-r requirements.txt", "coverage", "pytest>=7", "pytest-mock"]

**Sorted alphabetically:**

- ``dependency_groups``, ``allowlist_externals``, ``extras``, ``labels``, ``depends``

**Special handling for ``pass_env``:**

Replacement objects (inline tables like ``{ replace = "default", ... }``) are pinned to the start,
then string entries are sorted alphabetically:

.. code-block:: toml

    # Before
    pass_env = ["TERM", "CI", { replace = "default", ... }, "HOME"]

    # After
    pass_env = [{ replace = "default", ... }, "CI", "HOME", "TERM"]

**Arrays NOT sorted:**

- ``commands``, ``commands_pre``, ``commands_post`` — execution order matters
- ``base_python`` — first entry takes priority

Inline Table Key Reordering
~~~~~~~~~~~~~~~~~~~~~~~~~~~

Keys within inline tables are reordered into a consistent order based on the inline table's type. The type
is detected by the presence of a discriminator key:

- **``replace``** — ``replace`` → ``condition`` → ``of`` → ``env`` → ``key`` → ``name`` → ``pattern`` →
  ``then`` → ``else`` → ``default`` → ``extend`` → ``marker``
- **``prefix``** — ``prefix`` → ``start`` → ``stop``
- **``product``** — ``product`` → ``exclude``
- **``value``** — ``value`` → ``marker``

Keys not listed in the schema are appended at the end in their original order.

.. code-block:: toml

    # Before
    pass_env = [{ default = ".", replace = "default", extend = true }]
    env_list = [{ exclude = ["py38-django50"], product = ["py38", "py310", "django42", "django50"] }]

    # After
    pass_env = [{ replace = "default", default = ".", extend = true }]
    env_list = [{ product = ["py38", "py310", "django42", "django50"], exclude = ["py38-django50"] }]

This reordering applies to all inline tables in the file, including those nested inside arrays.

Other Tables
~~~~~~~~~~~~

Any unrecognized tables are preserved and reordered according to standard table ordering rules. Keys within
unknown tables are not reordered or normalized.
