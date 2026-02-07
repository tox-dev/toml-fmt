Overview
========

Apply a consistent format to your ``tox.toml`` file with comment support. See
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

:pypi:`tox-toml-fmt` is a CLI tool that needs a Python interpreter (version 3.10 or higher) to run. We recommend
either :pypi:`pipx` or :pypi:`uv` to install tox-toml-fmt into an isolated environment. This has the added benefit that
later you will be able to upgrade tox-toml-fmt without affecting other parts of the system. We provide a method for
``pip`` too here, but we discourage that path if you can:

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

You can use ``tox-toml-fmt`` as a Python module to format TOML content programmatically.

.. code-block:: python

    from tox_toml_fmt import run

    # Format a tox.toml file and return the exit code
    exit_code = run(["path/to/tox.toml"])

The ``run`` function accepts command-line arguments as a list and returns an exit code (0 for success, non-zero for
failure).

.. automodule:: tox_toml_fmt
   :members:

.. toctree::
   :hidden:

   self
   configuration
   formatting

See :doc:`configuration` for configuration options and command line interface.
