## Version number

major.minor.patch

## Release notes

These should be taken from the CHANGELOG, but any leading `### Heading` should be modified to be `Heading:`:

```text

```

## Checklist

- [ ] Branch is formatted `release/vX.Y.Z`
- [ ] Version is updated in Cargo.toml
- [ ] Benchmarks are updated
  - [ ] Run `scripts/bench` and commit the resultant plots/html files
  - [ ] Copy the timing output to BENCHMARKS.md
- [ ] CHANGELOG is updated w/ correct header and correct links
- [ ] CHANGELOG content is audited for correctness and clarity
- [ ] (after merge) Release commit is tagged as `vX.Y.Z` with the release notes in the annotation
- [ ] (after merge) Tag is pushed to Github
- [ ] (after merge) Crate is published
