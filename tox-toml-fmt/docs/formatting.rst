Formatting Rules
================

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

.. fmt-example::

    [env.test]
    description = 'Run tests'
    commands = ["echo \"hello\""]

Key Quotes
~~~~~~~~~~

TOML keys are normalized to the simplest valid form. Keys that are valid bare keys (containing only
``A-Za-z0-9_-``) have redundant quotes stripped. Single-quoted (literal) keys that require quoting are
converted to double-quoted (basic) strings with proper escaping. This applies to all keys: table headers,
key-value pairs, and inline table keys:

.. fmt-example::

    [env.'my env']
    "description" = "run tests"
    pass_env = [{ "else" = "no" }]

Backslashes and double quotes within literal keys are escaped during conversion.

Array Formatting
~~~~~~~~~~~~~~~~

Arrays are formatted based on line length, trailing comma presence, and comments. Short arrays stay on one line:

.. fmt-example::

    env_list = ["py312", "py313", "lint"]

Arrays that exceed ``column_width`` are expanded and get a trailing comma (shown here with a small ``column_width`` to
keep the example short):

.. fmt-example::
    :config: column_width=30

    [env.test]
    deps = ["pytest>=7", "coverage>=7", "tox>=4"]

A trailing comma forces the multiline format, even for an array that would otherwise fit on one line:

.. fmt-example::

    deps = ["pytest>=7",]

A comment on an entry also forces the multiline format. Here ``["pytest>=7", "coverage>=7"]`` would fit on one line,
but the comment keeps it expanded:

.. fmt-example::

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

Strings whose line runs past ``column_width`` are wrapped using TOML multiline basic strings with line-ending
backslashes (shown here with a small ``column_width``). The line is measured from the start of its key, so a long key
can be what pushes a value into wrapping; a key already wider than the column keeps its value on one line:

.. fmt-example::
    :config: column_width=40

    [env.test]
    description = "run the entire unit test suite with coverage"

Specific keys can be excluded from wrapping using ``skip_wrap_for_keys``. Patterns support wildcards
(e.g. ``*.commands`` skips wrapping for ``commands`` under any table).

Table Formatting
~~~~~~~~~~~~~~~~

Sub-tables can be formatted in two styles controlled by ``table_format``:

**Short format** (default, collapsed to dotted keys):

.. fmt-example::

    [env.test]
    description = "run tests"
    sub.value = 1

**Long format** (expanded to table headers):

.. fmt-example::
    :config: table_format=long

    [env.test]
    description = "run tests"
    sub.value = 1

Individual tables can override the default using ``expand_tables`` and ``collapse_tables``.

**Table spacing:**

By default, different table groups are separated by a blank line, while sub-tables within the same group are kept
compact. You can control this with ``sub_table_spacing`` and ``separate_root_table``. Each option takes a string of
``\n`` characters where each ``\n`` adds one blank line. For example, setting ``sub_table_spacing = "\n"`` adds a blank
line between sub-tables within the same environment.

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

Sub-tables within an environment (e.g. ``[env.test.sub]``) still follow the ``table_format`` setting, and are
ordered by the same environment key order as dotted keys: ones the order lists (such as ``set_env``) come first,
the rest follow alphabetically.

Comment Preservation
~~~~~~~~~~~~~~~~~~~~

All comments are preserved during formatting:

- **Inline comments** - Comments after a value on the same line stay with that value
- **Leading comments** - Comments on the line before an entry stay with the entry below
- **Block comments** - Multi-line comment blocks are preserved

**Inline comment alignment:**

Inline comments within arrays are aligned independently per array, based on that array's longest value:

.. fmt-example::

    deps = [
      "pytest", # testing
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

.. fmt-example::

    [env_run_base]
    description = "run the tests"
    # set_env = {A = "1"}

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

.. fmt-example::

    [env.test]
    deps = [
      # Group: runtime
      "requests",
      "click",
      # Group: testing
      "pytest-cov",
      "pytest",
    ]

Line Endings
~~~~~~~~~~~~

The formatter writes a file back with the line ending it already used, so a ``\r\n`` file stays ``\r\n`` and Git on
Windows does not flag it as modified. A file mixing both endings gets whichever one it uses more, with a tie going to
``\n``. Line endings alone never count as a change, so a file that is already formatted is left alone whichever ending
it uses. Output written to stdout always uses ``\n``.

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

.. fmt-example::

    envlist = ["py312", "py313"]
    minversion = "4.2"
    skipsdist = true

Full list: ``envlist`` → ``env_list``, ``toxinidir`` → ``tox_root``, ``toxworkdir`` → ``work_dir``,
``skipsdist`` → ``no_package``, ``isolated_build_env`` → ``package_env``, ``setupdir`` → ``package_root``,
``minversion`` → ``min_version``, ``ignore_basepython_conflict`` → ``ignore_base_python_conflict``

**Environment table aliases:**

.. fmt-example::

    [env_run_base]
    basepython = "python3.12"
    setenv.PYTHONPATH = "src"
    passenv = ["HOME"]

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

.. fmt-example::

    env_list = ["py312", "lint"]
    requires = ["tox>=4.2"]
    min_version = "4.2"

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

The keys of a ``set_env`` table keep the order the file gave them: tox reads that table in order, so a key written
after ``file`` overrides what the file said while one written before it does not.

.. fmt-example::

    [env_run_base]
    commands = ["pytest"]
    deps = ["pytest>=7"]
    description = "run tests"

``requires`` Normalization
~~~~~~~~~~~~~~~~~~~~~~~~~~

Dependencies in the root ``requires`` array are normalized per PEP 508 (canonical package names,
consistent spacing around specifiers) and sorted alphabetically by package name:

.. fmt-example::

    requires = ["tox >= 4.2", "tox-uv"]

``env_list`` Order
~~~~~~~~~~~~~~~~~~

The ``env_list`` array is written in a fixed order:

1. **Pinned environments**, in the order ``--pin-env`` names them
2. **CPython versions** (``py3.12``, ``py312``, ``3.12``), newest first
3. **PyPy versions** (``pypy3.10``, ``pypy310``), newest first
4. **Everything else** (``lint``, ``type``, ``docs``), by name

A compound name separated by ``-`` is placed by the first part of it that reads as one of those.

.. fmt-example::

    env_list = ["lint", "py38", "py312", "docs", "py310-django"]

An entry that generates environments rather than naming one, such as ``{ product = ... }``, names none of them, and
what it generates is read where it sits, so it holds the place the file gave it while the names around it move.

That order is also where each ``[env.NAME]`` table is written, so the file reads the way tox runs it. ``--pin-env``
(here ``fix,type``) moves both:

.. fmt-example::
    :config: pin_envs=fix,type

    env_list = ["lint", "fix", "type"]

    [env.lint]
    description = "lint"

    [env.fix]
    description = "fix"

    [env.type]
    description = "type"

See :doc:`configuration` for how to set ``pin-env`` via the config file or CLI.

``use_develop`` Upgrade
~~~~~~~~~~~~~~~~~~~~~~~

The legacy ``use_develop = true`` setting is automatically converted to the modern ``package = "editable"``
equivalent. If ``use_develop = false``, the key is left as-is. tox reads ``use_develop`` before ``package`` and
installs an editable package whatever ``package`` says, so a ``package`` key already there is given the mode the
environment ran with:

.. fmt-example::

    [env_run_base]
    use_develop = true

Array Sorting
~~~~~~~~~~~~~

Certain arrays within environment tables are sorted automatically:

**Sorted by canonical PEP 508 package name:**

- ``deps``: dependencies normalized and sorted by package name. ``constraints`` names the files tox hands to pip, so
  it is left as written.

pip reads this list the way it reads a requirements file, where a later ``--index-url`` replaces the one before it, so
a list holding anything but plain requirements keeps the order it names them in. Pip options and file references
(``-r``, ``-c``, ``-e``, ``--index-url``), local paths (``./``, ``../``, ``/``), URLs, artifact filenames (``.whl``,
``.zip``, ``.tar.gz`` and the other archive suffixes pip installs by name), and entries containing tox substitution
variables (``{tox_root}``, etc.) are each such an entry: they are left as written, and the list they sit in
is left in its order while every requirement beside them is still normalized:

.. fmt-example::

    [env_run_base]
    deps = ["Pytest >= 7", "-r requirements.txt", "coverage", "-e ./my-pkg[test]"]

**Sorted alphabetically:**

- ``dependency_groups``, ``allowlist_externals``, ``extras``, ``labels``, ``depends``

**Special handling for ``pass_env``:**

Replacement objects (inline tables like ``{ replace = "default", ... }``) are pinned to the start,
then string entries are sorted alphabetically:

.. fmt-example::

    [env.test]
    pass_env = ["TERM", "CI", { replace = "env", name = "PATH" }, "HOME"]

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

.. fmt-example::

    pass_env = [{ default = ".", replace = "default", extend = true }]
    env_list = [{ exclude = ["py312-django"], product = ["py312", "py313"] }]

This reordering applies to all inline tables in the file, including those nested inside arrays.

Other Tables
~~~~~~~~~~~~

Any unrecognized tables are preserved and reordered according to standard table ordering rules. Keys within
unknown tables are not reordered or normalized.
