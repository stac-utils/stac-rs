# Releasing

To create a new release:

1. Create a new branch that starts with `release/`, e.g. `release/v0.1.0`
2. Update the version in [Cargo.toml](./Cargo.toml)
3. Update [CHANGELOG.md](./CHANGELOG.md)
4. Open a pull request
5. If the pull request succeeds (it will run a special `cargo publish --dry-run` action), merge
6. Create an annotated tag pointing to the release, including the information from the changelog section corresponding to your release
7. Push your tag to Github
8. Publish your crate
