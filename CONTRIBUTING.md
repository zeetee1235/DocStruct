# Contributing

## Scope

- Keep changes focused and reviewable.
- Prefer small PRs over large mixed changes.

## Local Workflow

```bash
cargo build
cargo test
```

For OCR-related changes, run at least one fixture conversion:

```bash
./target/debug/docstruct convert tests/fixtures/test_document.pdf -o /tmp/docstruct_check --debug
```

## Pull Request Checklist

- Code builds and tests pass locally
- Behavior changes are documented in `README.md` or `docs/`
- New logic has at least one targeted test when practical
- No unrelated file churn in the PR

## Style

- Follow existing Rust and Python style in this repository.
- Keep comments short and technical.
- Avoid adding dependencies without clear need.
