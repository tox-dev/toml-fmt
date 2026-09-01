tox-toml-fmt
============

``tox-toml-fmt`` formats ``tox.toml`` files without discarding comments. It uses a small configuration surface to keep
results stable across projects. See the
`release history <https://github.com/tox-dev/toml-fmt/releases?q=tox-toml-fmt>`_ for version changes.

Install
-------

The command requires Python 3.10 or later. An isolated tool environment avoids dependency conflicts with the target
project.

.. tab:: uv

    .. code-block:: bash

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

    A user installation shares the interpreter's package set. Prefer ``uv`` or ``pipx`` when an operating system or
    package manager owns that interpreter.

Pre-commit
----------

Add the hook to ``.pre-commit-config.yaml`` and set ``rev`` to the required release:

.. code-block:: yaml

    - repo: https://github.com/tox-dev/tox-toml-fmt
      rev: ""
      hooks:
        - id: tox-toml-fmt

See :gh:`pre-commit/pre-commit` for installation and update commands.

Python API
----------

``run`` accepts command-line arguments and returns the process exit code:

.. code-block:: python

    from tox_toml_fmt import run

    exit_code = run(["path/to/tox.toml"])

.. automodule:: tox_toml_fmt
   :members:

.. toctree::
   :hidden:

   self
   configuration
   formatting

See :doc:`configuration` for settings and :doc:`formatting` for the rules applied to each table.
