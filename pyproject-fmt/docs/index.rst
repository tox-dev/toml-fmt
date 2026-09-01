pyproject-fmt
=============

``pyproject-fmt`` formats ``pyproject.toml`` files without discarding comments. It uses a small configuration surface
to keep results stable across projects. See the
`release history <https://github.com/tox-dev/toml-fmt/releases?q=pyproject-fmt>`_ for version changes.

Install
-------

The command requires Python 3.10 or later. An isolated tool environment avoids dependency conflicts with the target
project.

.. tab:: uv

    .. code-block:: bash

        uv tool install pyproject-fmt
        pyproject-fmt --help

.. tab:: pipx

    .. code-block:: bash

        python -m pip install pipx-in-pipx --user
        pipx install pyproject-fmt
        pyproject-fmt --help

.. tab:: pip

    .. code-block:: bash

        python -m pip install --user pyproject-fmt
        pyproject-fmt --help

    A user installation shares the interpreter's package set. Prefer ``uv`` or ``pipx`` when an operating system or
    package manager owns that interpreter.

Pre-commit
----------

Add the hook to ``.pre-commit-config.yaml`` and set ``rev`` to the required release:

.. code-block:: yaml

    - repo: https://github.com/tox-dev/pyproject-fmt
      rev: ""
      hooks:
        - id: pyproject-fmt

See :gh:`pre-commit/pre-commit` for installation and update commands.

Python API
----------

``run`` accepts command-line arguments and returns the process exit code:

.. code-block:: python

    from pyproject_fmt import run

    exit_code = run(["path/to/pyproject.toml"])

.. automodule:: pyproject_fmt
   :members:

.. toctree::
   :hidden:

   self
   configuration
   formatting

See :doc:`configuration` for settings and :doc:`formatting` for the rules applied to each table.
