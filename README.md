# rytest

rytest is a reasonably fast, somewhat Pytest compatible Python test runner.

Note that this is currently under heavy development, and will likely not work
for all but the simplest test suites.

## Why Another Test Runner?

What's wrong with pytest?  Well, nothing per se.  It's been a great test runner
for many years.  But it also tends to be quite slow, especially if you are
looking to do things like automatically rerun your tests across a large
codebase.

Rytest is an experiment to see if we can get a much faster test runner that is
still compatible with the pytest ecosystem. Some day we hope to build on that
and provide an evolution of the testing experience for python.

For now though you may be asking, how much faster is this really?  Well, we've
only built support for test collection so far, but it's substantially faster against a
couple of popular Python codebases.

![A benchmark comparing test collection in rytest against pytest in the Flask project, showing rytest is approximately twice as fast.](https://github.com/jimjkelly/rytest/blob/main/docs/benchmarks/flask-bench-0.1.0.png?raw=true)

![A benchmark comparing test collection in rytest against pytest in the FastAPI project, showing rytest is approximately five times faster.](https://github.com/jimjkelly/rytest/blob/main/docs/benchmarks/fastapi-bench-0.1.0.png?raw=true)

## Roadmap

We are releasing versions of rytest as we complete major feature groups.  The
following list is a rough roadmap of what we we expect to deliver:

- ✅ 0.1.0: Test collection.  This also offers an incomplete test execution framework.
- ⬜️ 0.2.0: Fixture Support.  This will ensure we have full fixture support, including test paramaterization.
- ⬜️ 0.3.0: Test Execution.  This will ensure we have a full test execution framework.
- ⬜️ 0.4.0: Test Reporting.  This will provide test reporting capabilties including coverage reporting.
- ⬜️ 0.5.0: Miscleneous.  This will include any other features we feel are necessary to make rytest a complete test runner.
- ⬜️ 1.0.0: Release.  This will be the first stable release of rytest, and is expected to be usable for most test suites.

This means that early released may not be suitable for production work, but we
hope people can pull it down, try it out, and provide feedback.

The roadmap to 2.0 will focus more on developing an interface unique to rytest,
more to come on that!

## Usage

The simple version is:

```bash
$ rytest tests
```

This will run tests in any python file in the `tests` directory that starts
with `test_`.

## Development

In order for maturin to build and link against python, you will need to ensure
there is a virtualenv available to it:

```bash
python3 -m venv .venv
```

To test against our local test fixtures, run:

```bash
cargo run -- tests -v
```

### Running the Test Suite

To run the test suite, run:

```bash
cargo test
```

We make use of the `insta` crate for snapshot testing. If you need to update
snapshots, run:

```bash
cargo install cargo-insta
cargo insta review
```

For more information, check out the [documentation](https://insta.rs/docs/cli/).

## Contributing

Before contributing code to this repository, recognize that you should run the
following to satisfy CI:

```bash
cargo fmt
cargo build
cargo clippy
cargo test
```

These will all be run in CI to validate your code.

## Thank You

We wanted to thank the Astral team for providing a great example of how to
manage a lot of the scaffold around a Rust/Python project, which has alllowed
us to focus on the core of rytest.