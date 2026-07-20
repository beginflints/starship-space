# Changelog

All notable changes to this project will be documented in this file.

This project follows a lightweight `Keep a Changelog` style and starts with `0.x` versioning.

## [Unreleased]

### Added

- Shared reinforcement mode with team-wide reserve lives, delayed player respawns, and a no-survivor fail condition.
- Role draft foundation with `Vanguard`, `Guardian`, and `Salvager` loadouts selected from the host lobby.
- Ship designer on mobile controllers with a budgeted grid editor (`Hull`, `Cockpit`, `Engine`, `Weapon`, `Wing`); custom designs render on the host screen.

### Changed

- Host HUD and market view now show team reinforcement status and per-player respawn countdowns.
- Mobile controller respawn overlay now shows the remaining respawn countdown from live game state.
- Role choices now change live gameplay through cooldown, HP, invincibility, and salvage-range modifiers.
- Mobile join flow now routes through the ship designer before entering the controller.

### Fixed

- Simultaneous enemy overlaps no longer apply repeated player-hit resolution in the same frame.

### Docs

- Reconciled `SUMMARY.md`, `IMPLEMENTATION_SPEC.md`, `TEAMWORK_MODES.md`, `Plan.md`, and `GAME_JOURNEY.md` with the shipped M3/M4/Ship Designer features (protocol tables, role stat table, Game Flow, debug keys, phase status, and stale proposal sections).
- Added repository workflow rules in `AGENTS.md`, including changelog-update expectations for future changes.

### Tests

- Unit test coverage for market purchase/offers logic (`apply_purchase`, `get_offers`) and WebSocket message (de)serialization (`ClientMsg`, `ShipPart`, `ServerMsg::State`).

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
