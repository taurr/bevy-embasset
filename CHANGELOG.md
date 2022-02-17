# Changelog
All notable changes to this project will be documented in this file.

Please read [CONTRIBUTING](./CONTRIBUTING.md#CHANGELOG).

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.1] - 2022-02-17
### Added
- Generated asset enums now implement `From<AssetEnum> for AssetPath`. This 
makes it possible to use the asset enum directly with `AssetServer.load(impl Into<AssetPath>)`.

## [0.3.0] - 2022-02-07
### Added
- `embasset_asset_ids` macro. Generating asset identification enum for use with the standard bevy `AssetIo`.
### Changed
- `embasset_assets` macro has been renamed to `assets`

## [0.2.0] - 2022-01-17
### Added
- `embasset_assets!` macro for defining:
    - an enum for identifying assets. The defined enum can be iterated, an lots more.
    - a struct implementing `AssetIo`, that'll contain the embedded assets. Needs to be added as a handler to a `EmbassetIo`.
### Removed
- feature `use-default-assetio` has been removed in favor of separate construction functions on `EmbassetIo`.
### Changed
- documentation updated to reflect the use of asset crates

## [0.1.2] - 2022-01-16
### Changed
- `AssetIoAlternative::fallback_on_err` is now a `#[must_use]`.
- CI: Now running `cargo check` etc. across Linux, Window and Mac on Github.
- CI: Checking the build automatically, once a week if nothing happens.

## [0.1.1] - 2022-01-15
Bugfixes/changes found in usage
### Removed
- dev-dependency on `bevy`
### Changed
- Embasset::new is new public.
    
    Enables the possibility to create Asset crates seperately from the game.
    
    Example can be found in the game [Bevoids](https://github.com/taurr/bevoids)


## [0.1.0] - 2022-01-15
Initial release of `bevy-embasset`


[Unreleased]: https://github.com/taurr/bevy-embasset/compare/0.4.1...HEAD
[0.4.1]: https://github.com/taurr/bevy-embasset/releases/tag/0.4.0
[0.3.0]: https://github.com/taurr/bevy-embasset/releases/tag/0.3.0
[0.2.0]: https://github.com/taurr/bevy-embasset/releases/tag/0.2.0
[0.1.2]: https://github.com/taurr/bevy-embasset/releases/tag/0.1.2
[0.1.1]: https://github.com/taurr/bevy-embasset/releases/tag/0.1.1
[0.1.0]: https://github.com/taurr/bevy-embasset/releases/tag/0.1.0