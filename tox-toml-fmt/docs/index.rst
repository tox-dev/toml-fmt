tox-toml-fmt
=============

Apply a consistent format to your ``pyproject.toml`` file with comment support. See
`changelog here <https://github.com/tox-dev/toml-fmt/blob/main/tox-toml-fmt/CHANGELOG.md>`_.


Philosophy
----------
This tool aims to be an *opinionated formatter*, with similar objectives to `black <https://github.com/psf/black>`_.
This means it deliberately does not support a wide variety of configuration settings. In return, you get consistency,
predictability, and smaller diffs.

Use
---

Via ``CLI``
~~~~~~~~~~~

:pypi:`tox-toml-fmt` is a CLI tool that needs a Python interpreter (version 3.10 or higher) to run. We recommend either
:pypi:`pipx` or :pypi:`uv` to install tox-toml-fmt into an isolated environment. This has the added benefit that later you'll
be able to upgrade tox-toml-fmt without affecting other parts of the system. We provide method for ``pip`` too here but we
discourage that path if you can:

.. tab:: uv

    .. code-block:: bash

        # install uv per https://docs.astral.sh/uv/#getting-started
        uv tool install tox-toml-fmt
        tox-toml-fmt --help


.. tab:: pipx

    .. code-block:: bash

        python -m pip install pipx-in-pipx --user
        pipx install tox-toml-fmt
        tox-toml-fmt --help

.. tab:: pip

    .. code-block:: bash

        python -m pip install --user tox-toml-fmt
        tox-toml-fmt --help

    You can install it within the global Python interpreter itself (perhaps as a user package via the
    ``--user`` flag). Be cautious if you are using a Python installation that is managed by your operating system or
    another package manager. ``pip`` might not coordinate with those tools, and may leave your system in an inconsistent
    state. Note, if you go down this path you need to ensure pip is new enough per the subsections below


Via ``pre-commit`` hook
~~~~~~~~~~~~~~~~~~~~~~~

See :gh:`pre-commit/pre-commit` for instructions, sample ``.pre-commit-config.yaml``:

.. code-block:: yaml

    - repo: https://github.com/tox-dev/tox-toml-fmt
      rev: "v1.0.0"
      hooks:
        - id: tox-toml-fmt

Via Python
~~~~~~~~~~

.. automodule:: tox_toml_fmt
   :members:

.. toctree::
   :hidden:

   self

Configuration via file
----------------------

The ``tox-toml-fmt`` table is used when present in the ``tox.toml`` file:

.. code-block:: toml

  [tox-toml-fmt]

  # after how many column width split arrays/dicts into multiple lines, 1 will force always
  column_width = 120

  # how many spaces use for indentation
  indent = 2

If not set they will default to values from the CLI, the example above shows the defaults.

Command line interface
----------------------
.. sphinx_argparse_cli::
  :module: tox_toml_fmt.__main__
  :func: _build_our_cli
  :prog: tox-toml-fmt
  :title:
