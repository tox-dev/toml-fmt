tox-toml-fmt
============

``tox-toml-fmt`` formats ``tox.toml`` files without discarding comments. It uses a small configuration surface to keep
results stable across projects. See the
`release history <https://github.com/tox-dev/toml-fmt/releases?q=tox-toml-fmt>`_ for version changes.

Install
-------

The command requires Python 3.10 or later. An isolated tool environment avoids dependency conflicts with the target
project.


    .. code-block:: bash

        uv tool install tox-toml-fmt
        tox-toml-fmt --help

Pre-commit
----------

Add the hook to ``.pre-commit-config.yaml`` and set ``rev`` to the required release:

.. code-block:: yaml

    - repo: https://github.com/tox-dev/tox-toml-fmt
      rev: ""
      hooks:
        - id: tox-toml-fmt

See `pre-commit/pre-commit <https://github.com/pre-commit/pre-commit>`_ for installation and update commands.

Python API
----------

``run`` accepts command-line arguments and returns the process exit code:

.. code-block:: python

    from tox_toml_fmt import run

    exit_code = run(["path/to/tox.toml"])

See the `configuration reference <https://tox-toml-fmt.readthedocs.io/en/latest/configuration.html>`_ for settings and
the `formatting reference <https://tox-toml-fmt.readthedocs.io/en/latest/formatting.html>`_ for the rules applied to each
table.

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

This reference separates file-wide rules from tox-specific ordering and normalization. See the
`configuration reference <https://tox-toml-fmt.readthedocs.io/en/latest/configuration.html>`_ for available settings.

General Formatting
------------------

These rules cover the complete ``tox.toml`` file.

String Quotes
~~~~~~~~~~~~~

Strings use double quotes unless the value contains one:

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

The formatter removes quotes from bare keys containing ``A-Za-z0-9_-``. Keys that need quotes use escaped double
quotes. The rule covers headers, assignments, and inline tables:

.. code-block:: toml

   # Before
   [env.'my env']
   "description" = "run tests"
   pass_env = [{ "else" = "no" }]

   # After
   [env."my env"]
   description = "run tests"
   pass_env = [ { else = "no" } ]

Conversion escapes backslashes and double quotes in literal keys.

Array Formatting
~~~~~~~~~~~~~~~~

Short arrays stay on one line:

.. code-block:: toml

   # Before
   env_list = ["py312", "py313", "lint"]

   # After
   env_list = [ "py313", "py312", "lint" ]

An array that exceeds ``column_width`` expands and gains a trailing comma:

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

A trailing comma retains multiline form at any width:

.. code-block:: toml

   # Before
   deps = ["pytest>=7",]

   # After
   deps = [
     "pytest>=7",
   ]

A member comment also retains multiline form:

.. code-block:: toml

   deps = [
     "pytest>=7",   # testing framework
     "coverage>=7",
   ]

An array uses multiline form when it has a trailing comma, exceeds ``column_width``, or contains a member comment.

String Wrapping
~~~~~~~~~~~~~~~

The formatter wraps a string that pushes its line past ``column_width``. Continuations account for the key prefix. A
key wider than the limit keeps its value on one line because wrapping cannot shorten that prefix:

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

``skip_wrap_for_keys`` excludes selected paths. For example, ``*.commands`` matches a ``commands`` key under any table.

Table Formatting
~~~~~~~~~~~~~~~~

``table_format`` selects a child-table shape.

Short form uses dotted keys:

.. code-block:: toml

   [env.test]
   description = "run tests"
   sub.value = 1

Long form uses headers:

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

A comment containing one valid assignment, such as ``# set_env = { A = "1" }``, represents a disabled field. The
formatter temporarily enables the assignment, formats it with its table, and restores the comment marker. This keeps
the field beside its active peers:

.. code-block:: toml

   # Before
   [env_run_base]
   description = "run the tests"
   # set_env = {A = "1"}

   # After
   [env_run_base]
   description = "run the tests"
   # set_env = { A = "1" }

Prose, multiline blocks, and commented headers remain ordinary comments. The check is structural: prose that parses as
one assignment receives disabled-field formatting. Rephrase such prose to avoid that interpretation. An
assignment wider than ``column_width`` also remains an ordinary comment.

Group Markers
~~~~~~~~~~~~~

An isolated ``# Group:`` comment divides an array or table into independent sort ranges. Group order and the marker
position stay fixed. Matching ignores case; trailing comments do not create boundaries.

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

Environment aliases:

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

Root keys follow this sequence:

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

The formatter applies PEP 508 spelling to root ``requires`` dependencies and sorts them by package name:

.. code-block:: toml

   # Before
   requires = ["tox >= 4.2", "tox-uv"]

   # After
   requires = [ "tox>=4.2", "tox-uv" ]

``env_list`` Order
~~~~~~~~~~~~~~~~~~

``env_list`` uses this order:

1. Environments named by ``--pin-env``, in argument order
2. CPython versions such as ``py3.12``, ``py312``, and ``3.12``, newest first
3. PyPy versions such as ``pypy3.10`` and ``pypy310``, newest first
4. Other names, alphabetically

A compound name takes the rank of its first recognized ``-``-separated part.

.. code-block:: toml

   # Before
   env_list = ["lint", "py38", "py312", "docs", "py310-django"]

   # After
   env_list = [ "py312", "py310-django", "py38", "docs", "lint" ]

An entry that generates environments rather than naming one, such as ``{ product = ... }``, names none of them, and
what it generates is read where it sits, so it holds the place the file gave it while the names around it move.

The same order controls ``[env.NAME]`` tables. ``--pin-env`` moves both the list entries and tables:

.. code-block:: toml

   # Before
   env_list = ["lint", "fix", "type"]

   [env.lint]
   description = "lint"

   [env.fix]
   description = "fix"

   [env.type]
   description = "type"

   # After
   env_list = [ "fix", "type", "lint" ]

   [env.fix]
   description = "fix"

   [env.type]
   description = "type"

   [env.lint]
   description = "lint"

See the `configuration reference <https://tox-toml-fmt.readthedocs.io/en/latest/configuration.html>`_ for how to set
``pin-env`` via the config file or CLI.

``use_develop`` Upgrade
~~~~~~~~~~~~~~~~~~~~~~~

The formatter converts ``use_develop = true`` to ``package = "editable"`` and retains ``use_develop = false``. When
both keys exist, tox lets ``use_develop`` determine the package mode, so the conversion applies that mode to
``package``:

.. code-block:: toml

   # Before
   [env_run_base]
   use_develop = true

   # After
   [env_run_base]
   package = "editable"

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

.. code-block:: toml

   # Before
   [env_run_base]
   deps = ["Pytest >= 7", "-r requirements.txt", "coverage", "-e ./my-pkg[test]"]

   # After
   [env_run_base]
   deps = [ "pytest>=7", "-r requirements.txt", "coverage", "-e ./my-pkg[test]" ]

Alphabetical order:

- ``dependency_groups``, ``allowlist_externals``, ``extras``, ``labels``, ``depends``

``pass_env``:

Replacement objects such as ``{ replace = "default", ... }`` lead the list, followed by sorted strings:

.. code-block:: toml

   # Before
   [env.test]
   pass_env = ["TERM", "CI", { replace = "env", name = "PATH" }, "HOME"]

   # After
   [env.test]
   pass_env = [ { replace = "env", name = "PATH" }, "CI", "HOME", "TERM" ]

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

.. code-block:: toml

   # Before
   pass_env = [{ default = ".", replace = "default", extend = true }]
   env_list = [{ exclude = ["py312-django"], product = ["py312", "py313"] }]

   # After
   env_list = [ { product = [ "py312", "py313" ], exclude = [ "py312-django" ] } ]
   pass_env = [ { replace = "default", default = ".", extend = true } ]

The rule includes inline tables nested in arrays.

Other Tables
~~~~~~~~~~~~

Unrecognized tables take their standard table position. Their keys and values retain input order and spelling.
