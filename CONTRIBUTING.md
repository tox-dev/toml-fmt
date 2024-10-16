# Contributing to toml-fmt

Thank you for your interest in contributing to toml-fmt! There are many ways to contribute, and we appreciate all of
them. As a reminder, all contributors are expected to follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## Development Setup

To work on the project:

1. Install Rust (preferably through [rustup](https://rustup.rs)).
2. Clone the repository.
3. Build the project and run the unit tests:

   ```bash
   # build a projects rust code
   cargo build -p common
   cargo build -p pyproject-fmt

   # run a projects rust test code
   cargo test -p common
   cargo test -p pyproject-fmt

   # for pyo3 objects use tox to run Python tests
   tox run -e 3.13
   tox run -e type
   ```

## License

By contributing to toml-fmt, you agree that your contributions will be licensed under the [MIT License](LICENSE).

Thank you for your contributions! If you have any questions or need further assistance, feel free to reach out via
GitHub issues.
