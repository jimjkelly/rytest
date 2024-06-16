# rytest

rytest is a reasonably fast, somewhat Pytest compatible Python test runner.

## Running Tests

The simple version is:

```bash
$ rytest tests/**/*.py 
```

This will run tests in any python file in the `tests` directory that starts with `test_`.

python 3.12

Goals:

Fast
pytest compatibility mode
inspiration from more ergonomic test runner UX such as Jest.

Structure:

Pipeline of:

gather
test
resulst


add out of the box tracing
- cardinality guard