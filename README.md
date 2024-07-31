# rytest

rytest is a reasonably fast, somewhat Pytest compatible Python test runner.

Note that this is under heavy development, and will not do anything for all
but the simplest of test suites.

## Running Tests

The simple version is:

```bash
$ rytest tests/**/*.py 
```

This will run tests in any python file in the `tests` directory that starts with `test_`.

## Development

To test against our local test fixtures, run:

```bash
cargo run -- tests/**/*.py -v
```

## Contributing

Before contributing code to this repository, recognize that you should run the following to satisfy CI:

```bash
cargo fmt
cargo build
cargo clippy
cargo test
```

These will all be run in CI to validate your code.

## Misc

python 3.12

Goals:

Fast
pytest compatibility mode
inspiration from more ergonomic test runner UX such as Jest.

Structure:

Pipeline of:

gather
test
results


add out of the box tracing
- cardinality guard