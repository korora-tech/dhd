# Changelog

All notable changes to this project will be documented in this file following
the [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format and
adhering to semantic versioning.

## [0.1.0] - YYYY-MM-DD

### Added
* Initial public release of **DHD**.
  * Cross-platform CLI for declarative dot-file management.
  * Optional TUI (ratatui) and GUI (Tauri) front-ends bundled in the same
    binary.
* Basic configuration file support via **confy**.
* Modular execution engine backed by a DAG with parallel execution (rayon).
