# Contributing to MON (Mycel Object Notation) Core

We welcome contributions to `mon-core`! By participating in this project, you agree to be a cool person.

## How to Contribute

### Reporting Bugs

If you find a bug, please open an issue on our [GitHub Issue Tracker](https://github.com/Mycel-Lang/mon-core/issues).
When reporting a bug, please include:

* A clear and concise description of the bug.
* Steps to reproduce the behavior.
* Expected behavior.
* Actual behavior.
* Any relevant error messages or stack traces.
* Your operating system and Rust version (`rustc --version`).

### Suggesting Enhancements

We'd love to hear your ideas for improving `mon-core`! Please open an issue on
our [GitHub Issue Tracker](https://github.com/Mycel-Lang/mon-core/issues) to suggest an enhancement. Describe:

* The problem you're trying to solve.
* How your suggested enhancement would help.
* Any alternative solutions you've considered. / How it exists in other languages

### Submitting Pull Requests (PRs)

1. **Fork the repository** and clone it to your local machine.
2. **Create a new branch** for your feature or bug fix: `git checkout -b feature/your-feature-name` or
   `git checkout -b bugfix/issue-number`.
3. **Make your changes.**
4. **Write tests** for your changes. Ensure existing tests pass.
5. **Run `cargo fmt`** to format your code.
6. **Run `cargo clippy --all-targets --features lsp -- -D warnings`** to lint your code. Address any warnings.
   - you can run 
8. **Update documentation** if your changes affect the public API or add new features.
9. **Commit your changes** with a clear and concise commit message.
10. **Push your branch** to your fork.
11. **Open a Pull Request** to the `main` branch of the `mycel-dot-org/mon` repository.

#### Pull Request Checklist:

*   [ ] Your code is formatted with `cargo fmt`.
*   [ ] Your code passes `cargo clippy`.
*   [ ] All tests pass (`cargo test --all-features`).
*   [ ] You have added new tests for your changes (if applicable).
*   [ ] Documentation has been updated (if applicable).
*   [ ] Your commit message is clear and descriptive.

Thank you for contributing!
