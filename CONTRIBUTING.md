# Contributing

Thanks for contributing to `jira-cli`.

## Development Setup

1. Install a current stable Rust toolchain.
2. Clone the repository.
3. Run:

```powershell
cargo fmt
cargo test
```

## Project Standards

- Keep user-facing command behavior covered by tests.
- Prefer boundary tests over tests that lock in internal implementation details.
- Keep modules cohesive. If a file starts owning multiple unrelated concepts, split it.
- Preserve cross-platform behavior for config paths, keyring usage, and shell output.

## Pull Requests

- Describe the behavior change and the motivation.
- Include or update tests when behavior changes.
- Keep documentation in sync with CLI behavior and flags.
- Run `cargo fmt` and `cargo test` before opening the PR.

## Releases

- Push a tag like `v0.1.2` to trigger the release workflow.
- The release workflow publishes GitHub Release assets and updates the Homebrew tap formula automatically.
- Maintainers must set the `HOMEBREW_TAP_TOKEN` repository secret with push access to `Rana-Faraz/homebrew-tap`, or the release workflow will fail at the tap update step.

## Reporting Issues

When filing bugs, include:

- Your operating system
- Rust version if building from source
- The command you ran
- The error output
- Whether you used stored credentials or `JIRA_TOKEN`
