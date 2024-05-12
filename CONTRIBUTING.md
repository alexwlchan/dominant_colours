# CONTRIBUTING

## Releasing a new version

1.  Bump the version number in `Cargo.toml`
2.  Create a new changelog entry for your version in `CHANGELOG.md`
3.  Commit your change
4.  Create a Git tag with your version number
5.  Push your new commit and Git tag to GitHub

GitHub Actions will then build and release a new version of the CLI tool for you.
