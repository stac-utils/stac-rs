# Releasing

## Checklist

1. Determine which packages you're releasing. While its not required, its
probably safest to release most or all of the packages at the same time -- use
your best judgement.
2. Create a release branch.
    - If you're only releasing one version (e.g. for a bugfix), name it `release/{package name}-{version}`, e.g. `release/stac-v1.2.3`.
    - If you're releasing multiple versions, just use the date, e.g. `release/2023-02-26`.
3. Determine each package's next version, and update their `Cargo.toml` files accordingly.
4. Scan each README for references to version numbers, and update any that are needed.
5. Update each package's CHANGELOG with a new section for the new version. Don't forget to update the links at the bottom, too.
6. Test the release with `cargo release`. By default, this does a dry-run, so it won't actually do anything. If you're _not_ releasing every package, specify the packages you want with the `-p` flag.
7. Use the normal pull request workflow to merge your branch.
8. Once merged, run `cargo release --execute` to do the release. Use the same `-p` flags as you did during the dry run.

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

We do not currently publish release notes (<https://github.com/gadomski/stac-rs/releases>).
This may change in the near future as **stac** (in particular) becomes more mature and is used by more downstream packages.
