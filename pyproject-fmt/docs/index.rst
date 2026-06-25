Overview
========

Apply a consistent format to your ``pyproject.toml`` file with comment support. See
`changelog here <https://github.com/tox-dev/toml-fmt/blob/main/pyproject-fmt/CHANGELOG.md>`_.


Philosophy
----------
This is an *opinionated formatter*, with the same objectives as `black <https://github.com/psf/black>`_: it offers few
configuration settings on purpose. In return you get consistency, predictability, and smaller diffs.

Use
---

Via ``CLI``
~~~~~~~~~~~

:pypi:`pyproject-fmt` is a CLI tool that needs Python 3.10 or higher to run. Install it into an isolated environment
with :pypi:`pipx` or :pypi:`uv`; that way you can upgrade pyproject-fmt later without disturbing the rest of your
system. A ``pip`` path follows for completeness, though we discourage it:

.. tab:: uv

    .. code-block:: bash

        # install uv per https://docs.astral.sh/uv/#getting-started
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

    You can install it into the global Python interpreter (as a user package via the ``--user`` flag). Take care if
    your Python is managed by the operating system or another package manager: ``pip`` may not coordinate with those
    tools and can leave your system inconsistent. On this path, make sure pip is new enough per the subsections below.


Via ``pre-commit`` hook
~~~~~~~~~~~~~~~~~~~~~~~

See :gh:`pre-commit/pre-commit` for instructions, sample ``.pre-commit-config.yaml``:

.. code-block:: yaml

    - repo: https://github.com/tox-dev/pyproject-fmt
      # Use the sha / tag you want to point at
      # or use `pre-commit autoupdate` to get the latest version
      rev: ""
      hooks:
        - id: pyproject-fmt

Via Python
~~~~~~~~~~

Call ``pyproject-fmt`` as a Python module to format TOML from your own code.

.. code-block:: python

    from pyproject_fmt import run

    # Format a pyproject.toml file and return the exit code
    exit_code = run(["path/to/pyproject.toml"])

The ``run`` function accepts command-line arguments as a list and returns an exit code (0 for success, non-zero for
failure).

.. automodule:: pyproject_fmt
   :members:

.. toctree::
   :hidden:

   self
   configuration
   formatting

See :doc:`configuration` for configuration options and command line interface.
