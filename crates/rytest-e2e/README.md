# rytest-e2e

Run tests against real repos.  This tool allows you to specify a repository,
a requirements file, and a test runner (usually either `rytest` or `pytest`)
and run tests against the repository.

```bash
cargo e2e https://github.com/fastapi/fastapi requirements-tests.txt rytest
```

```bash
cargo e2e https://github.com/fastapi/fastapi requirements-tests.txt pytest
```
