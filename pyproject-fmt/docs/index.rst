pyproject-fmt
=============

Apply a consistent format to your ``pyproject.toml`` file with comment support.
See `changelog here <https://github.com/tox-dev/pyproject-fmt/releases>`_.


Philosophy
----------
This tool aims to be an *opinionated formatter*, with similar objectives to
`black <https://github.com/psf/black>`_. This means it deliberately does not support
a wide variety of configuration settings. In return, you get consistency, predictability,
and smaller diffs.

Use
---

Via ``CLI``
~~~~~~~~~~~

Use `pipx <https://pipx.pypa.io/stable/installation/>`_ to install the project:

.. code-block:: shell

   pipx install pyproject-fmt


Via ``pre-commit`` hook
~~~~~~~~~~~~~~~~~~~~~~~

See :gh:`pre-commit/pre-commit` for instructions, sample ``.pre-commit-config.yaml``:

.. code-block:: yaml

    - repo: https://github.com/tox-dev/pyproject-fmt
      rev: "2.0.4"
      hooks:
        - id: pyproject-fmt

Via Python
~~~~~~~~~~

.. automodule:: pyproject_fmt
   :members:

.. toctree::
   :hidden:

   self

Configuration via file
----------------------

The ``tool.pyproject-fmt`` table is used when present in the ``pyproject.toml`` file:

.. code-block:: toml

  [tool.pyproject-fmt]

  # after how many column width split arrays/dicts into multiple lines, 1 will force always
  column_width = 120

  # how many spaces use for indentation
  indent = 2

  # if false will remove unnecessary trailing ``.0``'s from version specifiers
  keep_full_version = false

  # maximum Python version to use when generating version specifiers
  max_supported_python = "3.12"

If not set they will default to values from the CLI, the example above shows the defaults.

Command line interface
----------------------
.. sphinx_argparse_cli::
  :module: pyproject_fmt.__main__
  :func: _build_our_cli
  :prog: pyproject-fmt
  :title:

Python version classifiers
--------------------------

This tool will automatically generate the ``Programming Language :: Python :: 3.X`` classifiers for you. To do so it
needs to know the range of Python interpreter versions you support:

- The lower bound can be set via the ``requires-python`` key in the ``pyproject.toml`` configuration file (defaults to
  the oldest non end of line CPython version at the time of the release).
- The upper bound, by default, will assume the latest stable release of CPython at the time of the release, but can be
  changed via CLI flag or the config file.
