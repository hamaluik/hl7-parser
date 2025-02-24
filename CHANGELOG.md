# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2025-02-23

### Changed

- Rewrote the entire API from scratch
  -  After using the v0.2.0 API for some projects I really didn't like it;
     hopefully this one is better.
- Some minor performance improvements

### Added

- [jiff] support

## [0.2.0] - 2023-01-02

### Added

- Added optional [chrono] timestamp parsing

### Changed

- Made [time] an optional feature/dependency
- Disabled [serde] feature by default

## [0.1.0] - 2023-01-12

### Added

- Parse HL7v2 messages into a structure that can be queried
- Parse HL7v2 timestamps into [time] types
- [serde] derives on all data structures
- Decode HL7v2 encoded strings
- Locate a cursor within a message based on a character index
- Optional lenient parsing of segment separators (allow `\r\n`, `\n`, and `\r` to count as segment separators instead of just `\r`)

[serde]: https://crates.io/crates/serde
[time]: https://crates.io/crates/time
[chrono]: https://crates.io/crates/chrono
[jiff]: https://crates.io/crates/jiff

[unreleased]: https://github.com/olivierlacan/keep-a-changelog/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/olivierlacan/keep-a-changelog/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/olivierlacan/keep-a-changelog/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/olivierlacan/keep-a-changelog/releases/tag/v0.1.0
