# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- docs: Created a `CHANGELOG.md` file to document project updates
- tests: Added an integration testing suite `integration_test` to the `gmsol` tests
- sdk: Added the `gmsol::cli` module
- sdk: Added `SwitchboardPullOracleFactory` structure
- cli: Added support for Switchboard to the `order` subcommand

### Changed

- just: The `build-idls` recipient now builds IDLs that include docs
- programs: Replaced the `no-mock` feature with `mock`, meaning the default is "no-mock"
- programs: The `verify` instruction of the `mock-chainlink-verifier` program now will panic if it is not built with the `mock` feature enabled
- tests: Renamed `anchor_tests` testing suite to `anchor_test` in the `gmsol` tests
- sdk: Changed the arguments of `SwitchboardPullOracle::from_parts` function
- programs: Restricted the creation of instruction buffers so that only the executor wallet can be signer
- Updated dependencies:
  - `switchboard-on-demand`: `v0.3.4`

### Fixed

- programs: Fix the missing address validation when using Switchboard feeds.

## [0.3.0] - 2025-02-18

### Added

- Initial release
- Implemented core programs:
  - `gmsol-store`: Provide the protocol's core instructions, including permission management, market management, and core support for swaps and perpetual trading.
  - `gmsol-treasury`: Provides instructions for treasury management and implementing GT buyback.
- Provided SDK (`gmsol`) and other utility crates
- Provided a command-line interface (`gmsol`)

[unreleased]: https://github.com/gmsol-labs/gmx-solana/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/gmsol-labs/gmx-solana/releases/tag/v0.3.0
