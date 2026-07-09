Overview
========

Apply a consistent format to your ``tox.toml`` file with comment support. See
`changelog here <https://github.com/tox-dev/toml-fmt/blob/main/tox-toml-fmt/CHANGELOG.md>`_.


Recent Changes
~~~~~~~~~~~~~~~~

- ✨ feat(build): support free-threaded Python wheels by `@gaborbernat <https://github.com/gaborbernat>`_ in
  `#307 <https://github.com/tox-dev/toml-fmt/pull/307>`_
- 🐛 fix(common): skip empty tables in Tables::get by `@gaborbernat <https://github.com/gaborbernat>`_ in
  `#304 <https://github.com/tox-dev/toml-fmt/pull/304>`_
- Update Python dependencies by `@gaborbernat <https://github.com/gaborbernat>`_ in
  `#301 <https://github.com/tox-dev/toml-fmt/pull/301>`_
- Update Python dependencies by `@gaborbernat <https://github.com/gaborbernat>`_ in
  `#295 <https://github.com/tox-dev/toml-fmt/pull/295>`_
- Update Rust dependencies by `@gaborbernat <https://github.com/gaborbernat>`_ in
  `#296 <https://github.com/tox-dev/toml-fmt/pull/296>`_ <a id="1.9.2"></a>

Philosophy
----------
This is an *opinionated formatter*, with the same objectives as `black <https://github.com/psf/black>`_: it offers few
configuration settings on purpose. In return you get consistency, predictability, and smaller diffs.

Use
---

Via ``CLI``
~~~~~~~~~~~

`tox-toml-fmt <https://pypi.org/project/tox-toml-fmt>`_ is a CLI tool that needs Python 3.10 or higher to run. Install it into an isolated environment
with `pipx <https://pypi.org/project/pipx>`_ or `uv <https://pypi.org/project/uv>`_; that way you can upgrade tox-toml-fmt later without disturbing the rest of your
system. A ``pip`` path follows for completeness, though we discourage it:


    .. code-block:: bash

        # install uv per https://docs.astral.sh/uv/#getting-started
        uv tool install tox-toml-fmt
        tox-toml-fmt --help


Via ``pre-commit`` hook
~~~~~~~~~~~~~~~~~~~~~~~

See `pre-commit/pre-commit <https://github.com/pre-commit/pre-commit>`_ for instructions, sample ``.pre-commit-config.yaml``:

.. code-block:: yaml

    - repo: https://github.com/tox-dev/tox-toml-fmt
      rev: "v1.0.0"
      hooks:
        - id: tox-toml-fmt

Via Python
~~~~~~~~~~

Call ``tox-toml-fmt`` as a Python module to format TOML from your own code.

.. code-block:: python

    from tox_toml_fmt import run

    # Format a tox.toml file and return the exit code
    exit_code = run(["path/to/tox.toml"])

The ``run`` function accepts command-line arguments as a list and returns an exit code (0 for success, non-zero for
failure).


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

``tox-toml-fmt`` is an opinionated formatter, much like `black <https://github.com/psf/black>`_ is for Python code. It
keeps configuration minimal so every ``tox.toml`` lands on one standard format. That buys you:

- less time configuring tools
- smaller diffs when committing changes
- code reviews where formatting never comes up

A few options exist (``column_width``, ``indent``, ``table_format``, ``sub_table_spacing``, ``separate_root_table``),
but there are no dozens of toggles.

General Formatting
------------------

These rules apply uniformly across the entire ``tox.toml`` file.

String Quotes
~~~~~~~~~~~~~

All strings use double quotes by default. Single quotes are only used when the value contains double quotes:

.. code-block:: toml

   # Before
   [env.test]
   description = 'Run tests'
   commands = ["echo \"hello\""]

   # After
   [env.test]
   description = "Run tests"
   commands = [ 'echo "hello"' ]

Key Quotes
~~~~~~~~~~

TOML keys are normalized to the simplest valid form. Keys that are valid bare keys (containing only
``A-Za-z0-9_-``) have redundant quotes stripped. Single-quoted (literal) keys that require quoting are
converted to double-quoted (basic) strings with proper escaping. This applies to all keys: table headers,
key-value pairs, and inline table keys:

.. code-block:: toml

   # Before
   [env.'my env']
   "description" = "run tests"
   pass_env = [{ "else" = "no" }]

   # After
   [env."my env"]
   description = "run tests"
   pass_env = [ { else = "no" } ]

Backslashes and double quotes within literal keys are escaped during conversion.

Array Formatting
~~~~~~~~~~~~~~~~

Arrays are formatted based on line length, trailing comma presence, and comments. Short arrays stay on one line:

.. code-block:: toml

   # Before
   env_list = ["py312", "py313", "lint"]

   # After
   env_list = [ "py313", "py312", "lint" ]

Arrays that exceed ``column_width`` are expanded and get a trailing comma (shown here with a small ``column_width`` to
keep the example short):

.. code-block:: toml

   # Before
   [env.test]
   deps = ["pytest>=7", "coverage>=7", "tox>=4"]

   # After
   [env.test]
   deps = [
     "coverage>=7",
     "pytest>=7",
     "tox>=4",
   ]

A trailing comma forces the multiline format, even for an array that would otherwise fit on one line:

.. code-block:: toml

   # Before
   deps = ["pytest>=7",]

   # After
   deps = [
     "pytest>=7",
   ]

A comment on an entry also forces the multiline format. Here ``["pytest>=7", "coverage>=7"]`` would fit on one line,
but the comment keeps it expanded:

.. code-block:: toml

   deps = [
     "pytest>=7",   # testing framework
     "coverage>=7",
   ]

**Multiline formatting rules:**

An array becomes multiline when any of these conditions are met:

1. **Trailing comma present** - A trailing comma signals intent to keep multiline format
2. **Exceeds column width** - Arrays longer than ``column_width`` are expanded (and get a trailing comma added)
3. **Contains comments** - Arrays with inline or leading comments are always multiline

String Wrapping
~~~~~~~~~~~~~~~

Long strings that exceed ``column_width`` are wrapped using TOML multiline basic strings with line-ending backslashes
(shown here with a small ``column_width``):

.. code-block:: toml

   # Before
   [env.test]
   description = "run the entire unit test suite with coverage"

   # After
   [env.test]
   description = """\
     run the entire unit test suite with \
     coverage\
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

   # Before
   [env.test]
   description = "run tests"
   sub.value = 1

   # After
   [env.test]
   description = "run tests"
   [env.test.sub]
   value = 1

Individual tables can override the default using ``expand_tables`` and ``collapse_tables``.

**Table spacing:**

By default, different table groups are separated by a blank line, while sub-tables within the same group are kept
compact. You can control this with ``sub_table_spacing`` and ``separate_root_table``. Each option takes a string of
``\n`` characters where each ``\n`` adds one blank line. For example, setting ``sub_table_spacing = "\n"`` adds a blank
line between sub-tables within the same environment.


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

   # Before
   deps = [
     "pytest", # testing
     "pytest-cov",  # coverage
     "pytest-mock", # mocking
   ]

   # After
   deps = [
     "pytest",      # testing
     "pytest-cov",  # coverage
     "pytest-mock", # mocking
   ]

Disabled Keys
~~~~~~~~~~~~~

A commented-out line whose body is itself a single valid key-value (for example ``# set_env = { A = "1" }``) is treated
as a temporarily *disabled* field rather than free text. The formatter enables it for the duration of the pass, so it is
laid out and ordered together with the table it belongs to, then comments it out again on the way out. This keeps a
disabled key anchored to its entry instead of drifting to the next table, and formats the line the same way the enabled
key would be:

.. code-block:: toml

   # Before
   [env_run_base]
   description = "run the tests"
   # set_env = {A = "1"}

   # After
   [env_run_base]
   description = "run the tests"
   # set_env = { A = "1" }

Comments that are not a single valid key-value (prose, multi-line blocks, commented-out table headers like
``# [env.docs]``) are left untouched and follow the usual comment-preservation rules above. The heuristic is purely
structural, so a prose comment that *happens* to be valid TOML is reflowed too; if that matters, phrase the comment so it
does not parse as a key-value. Keys that would not fit on a single line within ``column_width`` are left as plain
comments.

Group Markers
~~~~~~~~~~~~~

By default the formatter reorders each array and table as a single unit, so any entry can move to its sorted position.
Mark a boundary with a standalone comment that starts with ``# Group:``: the formatter then sorts within each group,
holds the groups in their original order, and keeps the marker at the top of its group. Reach for this when related
entries belong together but should still be sorted.

Files without a ``# Group:`` marker format the same as before, so the feature stays opt-in. Case does not matter, so
``# group:`` works too. Only standalone comment lines count; the formatter ignores inline trailing comments.

.. code-block:: toml

   # Before
   [env.test]
   deps = [
     # Group: runtime
     "requests",
     "click",
     # Group: testing
     "pytest-cov",
     "pytest",
   ]

   # After
   [env.test]
   deps = [
     # Group: runtime
     "click",
     "requests",
     # Group: testing
     "pytest",
     "pytest-cov",
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
   min_version = "4.2"
   env_list = [ "py313", "py312" ]
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
   pass_env = [ "HOME" ]
   setenv.PYTHONPATH = "src"

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
   requires = [ "tox>=4.2" ]
   env_list = [ "py312", "lint" ]

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
   deps = [ "pytest>=7" ]
   commands = [ "pytest" ]

``requires`` Normalization
~~~~~~~~~~~~~~~~~~~~~~~~~~

Dependencies in the root ``requires`` array are normalized per PEP 508 (canonical package names,
consistent spacing around specifiers) and sorted alphabetically by package name:

.. code-block:: toml

   # Before
   requires = ["tox >= 4.2", "tox-uv"]

   # After
   requires = [ "tox>=4.2", "tox-uv" ]

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
   env_list = [ "py312", "py310-django", "py38", "docs", "lint" ]

Use ``--pin-env`` (here ``fix,type``) to pin specific environments to the start:

.. code-block:: toml

   # Before
   env_list = ["lint", "py312", "py313", "docs", "fix", "type"]

   # After
   env_list = [ "fix", "type", "py313", "py312", "docs", "lint" ]


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

- ``deps``, ``constraints``: dependencies normalized and sorted by package name

Pip file references (``-r``, ``-c``), editable installs (``-e``), local paths (``./``, ``../``,
``/``), and entries containing tox substitution variables (``{tox_root}``, etc.) are preserved as-is
without PEP 508 normalization, but still participate in sorting by their lowercased value:

.. code-block:: toml

   # Before
   [env_run_base]
   deps = ["Pytest >= 7", "-r requirements.txt", "coverage", "-e ./my-pkg[test]"]

   # After
   [env_run_base]
   deps = [ "-e ./my-pkg[test]", "-r requirements.txt", "coverage", "pytest>=7" ]

**Sorted alphabetically:**

- ``dependency_groups``, ``allowlist_externals``, ``extras``, ``labels``, ``depends``

**Special handling for ``pass_env``:**

Replacement objects (inline tables like ``{ replace = "default", ... }``) are pinned to the start,
then string entries are sorted alphabetically:

.. code-block:: toml

   # Before
   [env.test]
   pass_env = ["TERM", "CI", { replace = "env", name = "PATH" }, "HOME"]

   # After
   [env.test]
   pass_env = [ { replace = "env", name = "PATH" }, "CI", "HOME", "TERM" ]

**Arrays NOT sorted:**

- ``commands``, ``commands_pre``, ``commands_post``: execution order matters
- ``base_python``: first entry takes priority

Inline Table Key Reordering
~~~~~~~~~~~~~~~~~~~~~~~~~~~

Keys within inline tables are reordered into a consistent order based on the inline table's type. The type
is detected by the presence of a discriminator key:

- ``replace``: ``replace`` → ``condition`` → ``of`` → ``env`` → ``key`` → ``name`` → ``pattern`` →
  ``then`` → ``else`` → ``default`` → ``extend`` → ``marker``
- ``prefix``: ``prefix`` → ``start`` → ``stop``
- ``product``: ``product`` → ``exclude``
- ``value``: ``value`` → ``marker``

Keys not listed in the schema are appended at the end in their original order.

.. code-block:: toml

   # Before
   pass_env = [{ default = ".", replace = "default", extend = true }]
   env_list = [{ exclude = ["py312-django"], product = ["py312", "py313"] }]

   # After
   env_list = [ { product = [ "py312", "py313" ], exclude = [ "py312-django" ] } ]
   pass_env = [ { replace = "default", default = ".", extend = true } ]

This reordering applies to all inline tables in the file, including those nested inside arrays.

Other Tables
~~~~~~~~~~~~

Any unrecognized tables are preserved and reordered according to standard table ordering rules. Keys within
unknown tables are not reordered or normalized.
