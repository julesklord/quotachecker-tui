# Hygiene and Git Workflow

This project strictly follows the **[Your Organization] Development Bible**. Deviations will be met with silent judgment.

## Atomic Commits

The use of **Conventional Commits** is mandatory:
`<type>(<scope>): <subject>`

### Allowed Types

- `feat`: New functionality.
- `fix`: Bug correction.
- `docs`: Documentation changes.
- `style`: Visual changes (no logic).
- `refactor`: Code change that neither adds nor fixes anything. (Cleaning up your mess.)
- `chore`: Maintenance tasks, dependencies.

## Branch Workflow

- `main`: Production branch (linear history only).
- `feat/*`: Branches for new functionalities.
- `fix/*`: Branches for corrections.

**Banned:** `git push --force` to `main`. Don't even think about it.
