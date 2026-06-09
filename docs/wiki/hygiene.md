# Hygiene and Git Workflow

This project follows strict clean-code principles and atomic commit history.

## Atomic Commits

Use **Conventional Commits**: `<type>(<scope>): <subject>`

### Allowed Types

- `feat`: New functionality.
- `fix`: Bug correction.
- `docs`: Documentation changes in `docs/` or `README.md`.
- `style`: UI/UX changes (no logic).
- `refactor`: Code cleanup or improvements (e.g., Clippy fixes).
- `test`: Adding or updating tests.
- `chore`: Maintenance, dependencies, version bumps.

## Branch Workflow

- `main`: Protected. Only clean, tested code.
- `feat/*`: Feature development.
- `fix/*`: Bug fixes.

## Maintenance Rules

- **CHANGELOG.md**: Must be updated with every `feat` or `fix`.
- **VERSION**: Ensure the `VERSION` file matches `Cargo.toml`.
- **No Force Push**: Never force-push to `main`.
