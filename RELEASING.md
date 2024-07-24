# Releasing

## Checklist

1. Determine which package(s) you're releasing.
2. Determine the package's next version.
3. Create a release branch named `release/{package name}-{version}`, e.g. `release/stac-v1.2.3`.
4. Update the package's `Cargo.toml` file accordingly, and update the other packages' `Cargo.toml` if they depend on this package.
5. Scan the package's README for references to version numbers, and update any that are needed.
6. Update the package's CHANGELOG with a new section for the new version. Don't forget to update the links at the bottom, too.
7. If it's a breaking release, search for any deprecated functions that should be removed.
8. Test the release with `cargo release -p {package name}`. By default, this does a dry-run, so it won't actually do anything.
9. Use the normal pull request workflow to merge your branch.
10. Once merged, run `cargo release --execute` to do the release. Use the same `-p` flags as you did during the dry run.

## After-the-fact releases

Sometimes, the **main** branch has moved on, but you realize that you want to release a version of one of the packages from some previous commit, e.g. before a breaking change.
Follow the above workflow, with the following changes:

- Create your release branch from the point in history where you'd like to release from, not **main**.
- When your release pull request is approved, _don't_ merge it right away. Instead, run `cargo release --execute`. Then, manually merge your release branch into **main** -- you'll probably have to do some careful manual fixes to the CHANGELOGs. After you've merged, just push directly to **main**. This ensures we don't lose the tagged commit via an inadvertent Github rebase-and-merge.

## Semantic versioning and deprecation

All packages in **stac-rs** follow semantic versioning as best they can.
We do not currently require deprecation before removal, so features may disappear between breaking releases.
This may change in the future as the packages mature.

## Release notes

We do not currently publish release notes (<https://github.com/stac-utils/stac-rs/releases>).
This may change in the near future as **stac** (in particular) becomes more mature and is used by more downstream packages.
