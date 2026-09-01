Formatting Rules
================

This reference separates file-wide rules from tox-specific ordering and normalization. See :doc:`configuration` for
available settings.

General Formatting
------------------

These rules cover the complete ``tox.toml`` file.

String Quotes
~~~~~~~~~~~~~

Strings use double quotes unless the value contains one:

.. fmt-example::

    [env.test]
    description = 'Run tests'
    commands = ["echo \"hello\""]

Key Quotes
~~~~~~~~~~

The formatter removes quotes from bare keys containing ``A-Za-z0-9_-``. Keys that need quotes use escaped double
quotes. The rule covers headers, assignments, and inline tables:

.. fmt-example::

    [env.'my env']
    "description" = "run tests"
    pass_env = [{ "else" = "no" }]

Conversion escapes backslashes and double quotes in literal keys.

Array Formatting
~~~~~~~~~~~~~~~~

Short arrays stay on one line:

.. fmt-example::

    env_list = ["py312", "py313", "lint"]

An array that exceeds ``column_width`` expands and gains a trailing comma:

.. fmt-example::
    :config: column_width=30

    [env.test]
    deps = ["pytest>=7", "coverage>=7", "tox>=4"]

A trailing comma retains multiline form at any width:

.. fmt-example::

    deps = ["pytest>=7",]

A member comment also retains multiline form:

.. fmt-example::

    deps = [
      "pytest>=7",   # testing framework
      "coverage>=7",
    ]

An array uses multiline form when it has a trailing comma, exceeds ``column_width``, or contains a member comment.

String Wrapping
~~~~~~~~~~~~~~~

The formatter wraps a string that pushes its line past ``column_width``. Continuations account for the key prefix. A
key wider than the limit keeps its value on one line because wrapping cannot shorten that prefix:

.. fmt-example::
    :config: column_width=40

    [env.test]
    description = "run the entire unit test suite with coverage"

``skip_wrap_for_keys`` excludes selected paths. For example, ``*.commands`` matches a ``commands`` key under any table.

Table Formatting
~~~~~~~~~~~~~~~~

``table_format`` selects a child-table shape.

Short form uses dotted keys:

.. fmt-example::

    [env.test]
    description = "run tests"
    sub.value = 1

Long form uses headers:

.. fmt-example::
    :config: table_format=long

    [env.test]
    description = "run tests"
    sub.value = 1

``expand_tables`` and ``collapse_tables`` override individual paths. Root groups have one blank line between them.
Child tables stay adjacent unless ``sub_table_spacing`` adds a gap.

Environment tables retain ``[env.NAME]`` headers regardless of ``table_format``:

.. code-block:: toml

    [env.fix]
    description = "fix"

    [env.test]
    description = "test"

Child tables below an environment follow ``table_format``. Their headers use the same rank as dotted keys, followed by
unlisted names in alphabetical order.

Comment Preservation
~~~~~~~~~~~~~~~~~~~~

Comments move with the value or entry they describe. Within an array, trailing comments align against that array's
longest value:

.. fmt-example::

    deps = [
      "pytest", # testing
      "pytest-cov",  # coverage
      "pytest-mock", # mocking
    ]

Disabled Keys
~~~~~~~~~~~~~

A comment containing one valid assignment, such as ``# set_env = { A = "1" }``, represents a disabled field. The
formatter temporarily enables the assignment, formats it with its table, and restores the comment marker. This keeps
the field beside its active peers:

.. fmt-example::

    [env_run_base]
    description = "run the tests"
    # set_env = {A = "1"}

Prose, multiline blocks, and commented headers remain ordinary comments. The check is structural: prose that parses as
one assignment receives disabled-field formatting. Rephrase such prose to avoid that interpretation. An
assignment wider than ``column_width`` also remains an ordinary comment.

Group Markers
~~~~~~~~~~~~~

An isolated ``# Group:`` comment divides an array or table into independent sort ranges. Group order and the marker
position stay fixed. Matching ignores case; trailing comments do not create boundaries.

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

Output retains the input's line ending. Mixed files use the more frequent ending, with ties resolved to ``\n``. Stdout
uses ``\n``.

Table-Specific Handling
-----------------------

tox tables add the rules below.

Table Ordering
~~~~~~~~~~~~~~

The formatter writes tables in this order:

1. Root-level keys (``min_version``, ``requires``, ``env_list``, etc.)
2. ``[env_run_base]``
3. ``[env_pkg_base]``
4. ``[env_base.*]`` sections (shared base configurations)
5. ``[env.NAME]`` sections ordered by ``env_list`` if specified
6. Remaining ``[env.*]`` sections, alphabetically
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

Environments absent from ``env_list`` follow in alphabetical order.

Alias Normalization
~~~~~~~~~~~~~~~~~~~

The formatter renames legacy INI keys to their tox 4 TOML equivalents in the root, ``[env_run_base]``,
``[env_pkg_base]``, and ``[env.*]`` tables.

Root aliases:

.. fmt-example::

    envlist = ["py312", "py313"]
    minversion = "4.2"
    skipsdist = true

Full list: ``envlist`` → ``env_list``, ``toxinidir`` → ``tox_root``, ``toxworkdir`` → ``work_dir``,
``skipsdist`` → ``no_package``, ``isolated_build_env`` → ``package_env``, ``setupdir`` → ``package_root``,
``minversion`` → ``min_version``, ``ignore_basepython_conflict`` → ``ignore_base_python_conflict``

Environment aliases:

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

Root keys follow this sequence:

``min_version`` → ``requires`` → ``provision_tox_env`` → ``env_list`` → ``labels`` → ``base`` →
``package_env`` → ``package_root`` → ``no_package`` → ``skip_missing_interpreters`` →
``ignore_base_python_conflict`` → ``work_dir`` → ``temp_dir`` → ``tox_root``

.. fmt-example::

    env_list = ["py312", "lint"]
    requires = ["tox>=4.2"]
    min_version = "4.2"

Environment Key Ordering
~~~~~~~~~~~~~~~~~~~~~~~~~

Environment keys follow this sequence:

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

The formatter applies PEP 508 spelling to root ``requires`` dependencies and sorts them by package name:

.. fmt-example::

    requires = ["tox >= 4.2", "tox-uv"]

``env_list`` Order
~~~~~~~~~~~~~~~~~~

``env_list`` uses this order:

1. Environments named by ``--pin-env``, in argument order
2. CPython versions such as ``py3.12``, ``py312``, and ``3.12``, newest first
3. PyPy versions such as ``pypy3.10`` and ``pypy310``, newest first
4. Other names, alphabetically

A compound name takes the rank of its first recognized ``-``-separated part.

.. fmt-example::

    env_list = ["lint", "py38", "py312", "docs", "py310-django"]

An entry that generates environments rather than naming one, such as ``{ product = ... }``, names none of them, and
what it generates is read where it sits, so it holds the place the file gave it while the names around it move.

The same order controls ``[env.NAME]`` tables. ``--pin-env`` moves both the list entries and tables:

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

The formatter converts ``use_develop = true`` to ``package = "editable"`` and retains ``use_develop = false``. When
both keys exist, tox lets ``use_develop`` determine the package mode, so the conversion applies that mode to
``package``:

.. fmt-example::

    [env_run_base]
    use_develop = true

Array Sorting
~~~~~~~~~~~~~

Environment arrays use these policies.

PEP 508 package order:

- ``deps`` receives normalized dependency spelling and package-name order. ``constraints`` retains file order because
  it names files for pip.

pip reads this list the way it reads a requirements file, where a later ``--index-url`` replaces the one before it, so
a list holding anything but plain requirements keeps the order it names them in. Pip options and file references
(``-r``, ``-c``, ``-e``, ``--index-url``), local paths (``./``, ``../``, ``/``), URLs, artifact filenames (``.whl``,
``.zip``, ``.tar.gz`` and the other archive suffixes pip installs by name), and entries containing tox substitution
variables such as ``{tox_root}`` preserve list order. Plain requirements in the same list still receive normalized
spelling:

.. fmt-example::

    [env_run_base]
    deps = ["Pytest >= 7", "-r requirements.txt", "coverage", "-e ./my-pkg[test]"]

Alphabetical order:

- ``dependency_groups``, ``allowlist_externals``, ``extras``, ``labels``, ``depends``

``pass_env``:

Replacement objects such as ``{ replace = "default", ... }`` lead the list, followed by sorted strings:

.. fmt-example::

    [env.test]
    pass_env = ["TERM", "CI", { replace = "env", name = "PATH" }, "HOME"]

Preserved order:

- ``commands``, ``commands_pre``, ``commands_post``: execution order matters
- ``base_python``: first entry takes priority

Inline Table Key Reordering
~~~~~~~~~~~~~~~~~~~~~~~~~~~

A discriminator key selects the inline-table order:

- ``replace``: ``replace`` → ``condition`` → ``of`` → ``env`` → ``key`` → ``name`` → ``pattern`` →
  ``then`` → ``else`` → ``default`` → ``extend`` → ``marker``
- ``prefix``: ``prefix`` → ``start`` → ``stop``
- ``product``: ``product`` → ``exclude``
- ``value``: ``value`` → ``marker``

Unlisted keys follow in input order.

.. fmt-example::

    pass_env = [{ default = ".", replace = "default", extend = true }]
    env_list = [{ exclude = ["py312-django"], product = ["py312", "py313"] }]

The rule includes inline tables nested in arrays.

Other Tables
~~~~~~~~~~~~

Unrecognized tables take their standard table position. Their keys and values retain input order and spelling.
