# Version number

major.minor.patch

## Release notes

These should be taken from the CHANGELOG, but any leading `### Heading` should be modified to be `Heading:`:

```text

```

## Checklist

- [ ] Branch is formatted `release/vX.Y.Z`
- [ ] Version is updated in Cargo.toml
- [ ] Version is updated in README.md examples
- [ ] CHANGELOG is updated w/ correct header and correct links
- [ ] CHANGELOG content is audited for correctness and clarity
- [ ] (after merge, if necessary) Run `cargo install cargo-release`
- [ ] (after merge) Run `cargo release`
