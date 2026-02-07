Formatting Rules
================

``pyproject-fmt`` is an opinionated formatter, much like `black <https://github.com/psf/black>`_ is for Python code.
The tool intentionally provides minimal configuration options because the goal is to establish a single standard format
that all ``pyproject.toml`` files follow.

**Benefits of this approach:**

- Less time configuring tools
- Smaller diffs when committing changes
- Easier code reviews since formatting is never a question

While a few key options exist (``column_width``, ``indent``, ``table_format``), the tool does not expose dozens of
toggles. You get what the maintainers have chosen to be the right balance of readability, consistency, and usability.

General Formatting
------------------

These rules apply uniformly across the entire ``pyproject.toml`` file.

Table Ordering
~~~~~~~~~~~~~~

Tables are reordered into a consistent structure:

1. ``[build-system]``
2. ``[project]``
3. ``[dependency-groups]``
4. ``[tool.*]`` sections in the order:

   1. Build backends: ``poetry``, ``poetry-dynamic-versioning``, ``pdm``, ``setuptools``, ``distutils``,
      ``setuptools_scm``, ``hatch``, ``flit``, ``scikit-build``, ``meson-python``, ``maturin``, ``whey``,
      ``py-build-cmake``, ``sphinx-theme-builder``, ``uv``
   2. Builders: ``cibuildwheel``, ``nuitka``
   3. Linters/formatters: ``autopep8``, ``black``, ``ruff``, ``isort``, ``flake8``, ``pycln``, ``nbqa``,
      ``pylint``, ``repo-review``, ``codespell``, ``docformatter``, ``pydoclint``, ``tomlsort``,
      ``check-manifest``, ``check-sdist``, ``check-wheel-contents``, ``deptry``, ``pyproject-fmt``, ``typos``
   4. Testing: ``pytest``, ``pytest_env``, ``pytest-enabler``, ``coverage``
   5. Task runners: ``doit``, ``spin``, ``tox``
   6. Release tools: ``bumpversion``, ``jupyter-releaser``, ``tbump``, ``towncrier``, ``vendoring``
   7. Type checkers: ``mypy``, ``pyrefly``, ``pyright``, ``ty``, ``django-stubs``
   8. Any other ``tool.*`` in alphabetical order

5. Any other tables (alphabetically)

String Quotes
~~~~~~~~~~~~~

All strings use double quotes by default. Single quotes are only used when the value contains double quotes:

.. code-block:: toml

    # Before
    name = 'my-package'
    description = "He said \"hello\""

    # After
    name = "my-package"
    description = 'He said "hello"'

Array Formatting
~~~~~~~~~~~~~~~~

Arrays are formatted based on line length, trailing comma presence, and comments:

.. code-block:: toml

    # Short arrays stay on one line
    keywords = ["python", "toml"]

    # Long arrays that exceed column_width are expanded and get a trailing comma
    dependencies = [
        "requests>=2.28",
        "click>=8.0",
    ]

    # Trailing commas signal intent to keep multiline format
    classifiers = [
        "Development Status :: 4 - Beta",
    ]

    # Arrays with comments are always multiline
    lint.ignore = [
        "E501",  # Line too long
        "E701",
    ]

**Multiline formatting rules:**

An array becomes multiline when any of these conditions are met:

1. **Trailing comma present** - A trailing comma signals intent to keep multiline format
2. **Exceeds column width** - Arrays longer than ``column_width`` are expanded (and get a trailing comma added)
3. **Contains comments** - Arrays with inline or leading comments are always multiline

Table Formatting
~~~~~~~~~~~~~~~~

Sub-tables can be formatted in two styles controlled by ``table_format``:

**Short format** (collapsed to dotted keys):

.. code-block:: toml

    [project]
    urls.homepage = "https://example.com"
    urls.repository = "https://github.com/example/project"

**Long format** (expanded to table headers):

.. code-block:: toml

    [project.urls]
    homepage = "https://example.com"
    repository = "https://github.com/example/project"

See :doc:`configuration` for how to control this behavior.

Comment Preservation
~~~~~~~~~~~~~~~~~~~~

All comments are preserved during formatting:

- **Inline comments** - Comments after a value on the same line stay with that value
- **Leading comments** - Comments on the line before an entry stay with the entry below
- **Block comments** - Multi-line comment blocks are preserved

**Inline comment alignment:**

Inline comments within arrays are aligned independently per array, based on that array's longest value:

.. code-block:: toml

    # Before - comments at inconsistent positions
    lint.ignore = [
      "COM812", # Conflict with formatter
      "CPY", # No copyright statements
      "ISC001",   # Another rule
    ]

    # After - comments align to longest value in this array
    lint.ignore = [
      "COM812",  # Conflict with formatter
      "CPY",     # No copyright statements
      "ISC001",  # Another rule
    ]

Table-Specific Handling
-----------------------

Beyond general formatting, each table has specific key ordering and value normalization rules.

``[build-system]``
~~~~~~~~~~~~~~~~~~

**Key ordering:** ``build-backend`` → ``requires`` → ``backend-path``

**Value normalization:**

- ``requires``: Dependencies normalized per PEP 508 and sorted alphabetically by package name
- ``backend-path``: Entries sorted alphabetically

.. code-block:: toml

    # Before
    [build-system]
    requires = ["setuptools >= 45", "wheel"]
    build-backend = "setuptools.build_meta"

    # After
    [build-system]
    build-backend = "setuptools.build_meta"
    requires = ["setuptools>=45", "wheel"]

``[project]``
~~~~~~~~~~~~~

**Key ordering:**

Keys are reordered in this sequence: ``name`` → ``version`` → ``import-names`` → ``import-namespaces`` →
``description`` → ``readme`` → ``keywords`` → ``license`` → ``license-files`` → ``maintainers`` → ``authors`` →
``requires-python`` → ``classifiers`` → ``dynamic`` → ``dependencies`` → ``optional-dependencies`` → ``urls`` →
``scripts`` → ``gui-scripts`` → ``entry-points``

**Field normalizations:**

``name``
    Converted to canonical format (lowercase with hyphens): ``My_Package`` → ``my-package``

``description``
    Whitespace normalized: multiple spaces collapsed, consistent spacing after periods.

``license``
    License expression operators (``and``, ``or``, ``with``) uppercased: ``MIT or Apache-2.0`` → ``MIT OR Apache-2.0``

``requires-python``
    Whitespace removed: ``>= 3.9`` → ``>=3.9``

``keywords``
    Deduplicated (case-insensitive) and sorted alphabetically.

``dynamic``
    Sorted alphabetically.

``import-names`` / ``import-namespaces``
    Semicolon spacing normalized (``foo;bar`` → ``foo; bar``), entries sorted alphabetically.

``classifiers``
    Deduplicated and sorted alphabetically.

``authors`` / ``maintainers``
    Sorted by name, then email. Keys within each entry ordered: ``name`` → ``email``.

**Dependency normalization:**

All dependency arrays (``dependencies``, ``optional-dependencies.*``) are:

- Normalized per PEP 508: spaces removed, redundant ``.0`` suffixes stripped (unless ``keep_full_version = true``)
- Sorted alphabetically by canonical package name

.. code-block:: toml

    # Before
    dependencies = ["requests >= 2.0.0", "click~=8.0"]

    # After
    dependencies = ["click>=8", "requests>=2"]

**Optional dependencies extra names:**

Extra names are normalized to lowercase with hyphens:

.. code-block:: toml

    # Before
    [project.optional-dependencies]
    Dev_Tools = ["pytest"]

    # After
    [project.optional-dependencies]
    dev-tools = ["pytest"]

**Python version classifiers:**

Classifiers for Python versions are automatically generated based on ``requires-python`` and
``max_supported_python``. Disable with ``generate_python_version_classifiers = false``.

.. code-block:: toml

    # With requires-python = ">=3.10" and max_supported_python = "3.14"
    classifiers = [
        "Programming Language :: Python :: 3 :: Only",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Programming Language :: Python :: 3.13",
        "Programming Language :: Python :: 3.14",
    ]

**Entry points:**

Inline tables within ``entry-points`` are expanded to dotted keys:

.. code-block:: toml

    # Before
    entry-points.console_scripts = { mycli = "mypackage:main" }

    # After
    entry-points.console_scripts.mycli = "mypackage:main"

**Authors/maintainers formatting:**

Contact information can be formatted as inline tables or expanded array of tables:

.. code-block:: toml

    # Short format (inline)
    authors = [{ name = "Alice", email = "alice@example.com" }]

    # Long format (array of tables)
    [[project.authors]]
    name = "Alice"
    email = "alice@example.com"

Controlled by ``table_format``, ``expand_tables``, and ``collapse_tables``.

``[dependency-groups]``
~~~~~~~~~~~~~~~~~~~~~~~

**Key ordering:** ``dev`` → ``test`` → ``type`` → ``docs`` → others alphabetically

**Value normalization:**

- All dependencies normalized per PEP 508
- Sorted: regular dependencies first, then ``include-group`` entries

.. code-block:: toml

    # Before
    [dependency-groups]
    dev = [{ include-group = "test" }, "ruff>=0.4", "mypy>=1"]

    # After
    [dependency-groups]
    dev = ["mypy>=1", "ruff>=0.4", { include-group = "test" }]

``[tool.ruff]``
~~~~~~~~~~~~~~~

**Key ordering:**

Keys are reordered in a logical sequence:

1. Global settings: ``required-version`` → ``extend`` → ``target-version`` → ``line-length`` → ``indent-width`` →
   ``tab-size``
2. Path settings: ``builtins`` → ``namespace-packages`` → ``src`` → ``include`` → ``extend-include`` → ``exclude`` →
   ``extend-exclude`` → ``force-exclude`` → ``respect-gitignore``
3. Behavior flags: ``preview`` → ``fix`` → ``unsafe-fixes`` → ``fix-only`` → ``show-fixes`` → ``show-source``
4. Output settings: ``output-format`` → ``cache-dir``
5. ``format.*`` keys
6. ``lint.*`` keys: ``select`` → ``extend-select`` → ``ignore`` → ``extend-ignore`` → ``per-file-ignores`` →
   ``fixable`` → ``unfixable`` → plugin configurations

**Sorted arrays:**

Arrays are sorted alphabetically using natural ordering (``RUF1`` < ``RUF9`` < ``RUF10``):

.. code-block:: toml

    # These arrays are sorted:
    lint.select = ["E", "F", "I", "RUF"]
    lint.ignore = ["E501", "E701"]

    # Per-file-ignores values are also sorted:
    lint.per-file-ignores."tests/*.py" = ["D103", "S101"]

**Sorted array keys:**

Top-level:
  ``exclude``, ``extend-exclude``, ``include``, ``extend-include``, ``builtins``, ``namespace-packages``, ``src``

Format:
  ``format.exclude``

Lint:
  ``select``, ``extend-select``, ``ignore``, ``extend-ignore``, ``fixable``, ``extend-fixable``, ``unfixable``,
  ``extend-safe-fixes``, ``extend-unsafe-fixes``, ``external``, ``task-tags``, ``exclude``, ``typing-modules``,
  ``allowed-confusables``, ``logger-objects``

Per-file patterns:
  ``lint.per-file-ignores.*``, ``lint.extend-per-file-ignores.*``

Plugin arrays:
  ``lint.flake8-bandit.hardcoded-tmp-directory``, ``lint.flake8-bandit.hardcoded-tmp-directory-extend``,
  ``lint.flake8-boolean-trap.extend-allowed-calls``, ``lint.flake8-bugbear.extend-immutable-calls``,
  ``lint.flake8-builtins.builtins-ignorelist``, ``lint.flake8-gettext.extend-function-names``,
  ``lint.flake8-gettext.function-names``, ``lint.flake8-import-conventions.banned-from``,
  ``lint.flake8-pytest-style.raises-extend-require-match-for``, ``lint.flake8-pytest-style.raises-require-match-for``,
  ``lint.flake8-self.extend-ignore-names``, ``lint.flake8-self.ignore-names``,
  ``lint.flake8-tidy-imports.banned-module-level-imports``, ``lint.flake8-type-checking.exempt-modules``,
  ``lint.flake8-type-checking.runtime-evaluated-base-classes``,
  ``lint.flake8-type-checking.runtime-evaluated-decorators``, ``lint.isort.constants``,
  ``lint.isort.default-section``, ``lint.isort.extra-standard-library``, ``lint.isort.forced-separate``,
  ``lint.isort.no-lines-before``, ``lint.isort.required-imports``, ``lint.isort.single-line-exclusions``,
  ``lint.isort.variables``, ``lint.pep8-naming.classmethod-decorators``, ``lint.pep8-naming.extend-ignore-names``,
  ``lint.pep8-naming.ignore-names``, ``lint.pep8-naming.staticmethod-decorators``,
  ``lint.pydocstyle.ignore-decorators``, ``lint.pydocstyle.property-decorators``, ``lint.pyflakes.extend-generics``,
  ``lint.pylint.allow-dunder-method-names``, ``lint.pylint.allow-magic-value-types``

Other Tables
~~~~~~~~~~~~

Any unrecognized tables are preserved and reordered according to standard table ordering rules. Keys within unknown
tables are not reordered or normalized.
