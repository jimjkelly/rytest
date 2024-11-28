# Releasing a New Version of Rytest

This is internal documentation for maintainers of Rytest. If you are not a maintainer, you can ignore this document.

## 1. Update the Version Number

Update the version number in `Cargo.toml` to the new version number. This should be a semver-compatible version number.

## 2. Lock the Version of Rytest in the `Cargo.toml` File

Run `cargo lock` to update the `Cargo.lock` file with the new version of Rytest.

## 3. Make a Pull Request

With the version number updated, make a pull request with the changes. You can include any other related changes.
Merge this PR.

## 4. Create a New Release

Go to the [Releases](https:/.github.com/jimjkelly/rytest/releases) page on GitHub and click "Draft a new release".
In the "Choose a tag" dropdown, enter the new tag `v<the.new.version>` and select "Create new tag on publish" when the option appears.
Enter the release title in the form `v<the.new.version>`, and add a useful description.  Click the "Generate release notes" button.
Finally, click the "Publish release" button.
