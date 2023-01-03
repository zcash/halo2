# Feature development

Sometimes feature development can require iterating on a design over time. It can be
useful to start using features in downstream crates early on to gain experience with the
APIs and functionality, that can feed back into the feature's design prior to it being
stabilised. To enable this, we follow a three-stage `nightly -> beta -> stable`
development pattern inspired by (but not identical to) the Rust compiler.

## Feature flags

Each unstabilised feature has a default-off feature flag that enables it, of the form
`unstable-*`. The stable API of the crates must not be affected when the feature flag is
disabled, except for specific complex features that will be considered on a case-by-case
basis.

Two meta-flags are provided to enable all features at a particular stabilisation level:
- `beta` enables all features at the "beta" stage (and implicitly all features at the
  "stable" stage).
- `nightly` enables all features at the "beta" and "nightly" stages (and implicitly all
  features at the "stable" stage), i.e. all features are enabled.
- When neither flag is enabled (and no feature-specific flags are enabled), then in effect
  only features at the "stable" stage are enabled.

## Feature workflow

- If the maintainers have rough consensus that an experimental feature is generally
  desired, its initial implementation can be merged into the codebase optimistically
  behind a feature-specific feature flag with a lower standard of review. The feature's
  flag is added to the `nightly` feature flag set.
  - The feature will become usable by downstream published crates in the next general
    release of the `halo2` crates.
  - Subsequent development and refinement of the feature can be performed in-situ via
    additional PRs, along with additional review.
  - If the feature ends up having bad interactions with other features (in particular,
    already-stabilised features), then it can be removed later without affecting the
    stable or beta APIs.
- Once the feature has had sufficient review, and is at the point where a `halo2` user
  considers it production-ready (and is willing or planning to deploy it to production),
  the feature's feature flag is moved to the `beta` feature flag set.
- Once the feature has had review equivalent to the stable review policy, and there is
  rough consensus that the feature is useful to the wider `halo2` userbase, the feature's
  feature flag is removed and the feature becomes part of the main maintained codebase.

> For more complex features, the above workflow might be augmented with `beta` and
> `nightly` branches; this will be figured out once a feature requiring this is proposed
> as a candidate for inclusion.

## In-progress features

| Feature flag | Stage | Notes |
| --- | --- | --- |
| `unstable-sha256-gadget` | `nightly` | The SHA-256 gadget and chip.
