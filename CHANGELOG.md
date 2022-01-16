# Changelog
All notable changes to this project will be documented in this file.

Please read [CONTRIBUTING](./CONTRIBUTING.md#CHANGELOG).

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- `AssetIoAlternative::fallback_on_err` is now a `#[must_use]`.
- CI: Now running `cargo check` etc. across Linux, Window and Mac on Github.
- CI: Checking the build automatically, once a week if nothing happens.

### Changed
### Removed

## [0.1.1] - 2022-01-15
Bugfixes/changes found in usage
- Removed dev-dependency on `bevy`
- Embasset::new is new public.
    
    Enables the possibility to create Asset crates seperately from the game.
    
    Example can be found in the game [Bevoids](https://github.com/taurr/bevoids)

## [0.1.0] - 2022-01-15
Initial release of `bevy-embasset`

[Unreleased]: https://github.com/taurr/bevy-embasset/compare/0.1.1...HEAD
[0.1.1]: https://github.com/taurr/bevy-embasset/releases/tag/0.1.1
[0.1.0]: https://github.com/taurr/bevy-embasset/releases/tag/0.1.0