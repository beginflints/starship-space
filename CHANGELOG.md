# Changelog

All notable changes to this project will be documented in this file.

This project follows a lightweight `Keep a Changelog` style and starts with `0.x` versioning.

## [Unreleased]

### Added

- Role draft foundation with `Vanguard`, `Guardian`, and `Salvager` loadouts selected from the host lobby.

### Changed

- Role choices now change live gameplay through cooldown, HP, invincibility, and salvage-range modifiers.

### Fixed

- Placeholder for bug fixes and stability work not released yet.

### Docs

- Added repository workflow rules in `AGENTS.md`, including changelog-update expectations for future changes.

## [0.1.0] - 2026-03-12

### Added

- Initial playable `Starship Space` prototype with local Wi-Fi multiplayer host/client flow.
- Core phase loop: `Lobby`, `Playing`, `Market`, and `GameOver`.
- Enemy set with `Basic`, `Fast`, and `Tank` variants.
- Item drop and market upgrade systems.
- Convoy mode MVP with convoy core HUD and fail condition.
- Game over recap screen with quick replay, lobby return, and mode-switch shortcuts.
- Lobby roster polish with player name and ready-state display.

### Changed

- Market state sync now refreshes player coins and affordability during the market phase.

### Fixed

- Join/disconnect lifecycle and player-name propagation from mobile controllers to the host screen.
