# TASKS.md

## Active Tasks
- [x] Task 1 — Design and implement unified URL resolver (point #4)
  - Goal: Accept a story URL, auto-detect supported site, and fetch normalized story data without requiring `--site`.
  - Steps:
    - [x] 1.1 Add robust URL parsing + site detection in `nexus-core` (host + path based).
    - [x] 1.2 Add resolver API that routes URL -> correct site implementation.
    - [x] 1.3 Integrate resolver into CLI (`get-story-data-from-url`) with optional `--site` override.
    - [x] 1.4 Improve resolver-related error messages for invalid/unsupported URLs.
    - [x] 1.5 Run build check and verify with sample URLs.
  - Owner: assistant
  - Priority: high
  - Due: ASAP
  - Link: local implementation complete

- [x] Task 2 — Research and select Rust TUI technology (DuckDuckGo)
  - Goal: Pick the best-supported stack for a text UI scraper reader/downloader.
  - Steps:
    - [x] 2.1 Search DuckDuckGo for current Rust TUI options.
    - [x] 2.2 Compare ecosystem maturity and ergonomics.
    - [x] 2.3 Decide stack and record rationale.
  - Decision:
    - Selected `ratatui` + `crossterm`.
    - Rationale: current ecosystem momentum, official docs/tutorial coverage, broad usage examples, async-template support.
  - Owner: assistant
  - Priority: high
  - Due: ASAP
  - Link: local implementation complete

- [x] Task 3 — Create separate Rust crate for TUI (`nexus-tui`)
  - Goal: Add a dedicated crate to input URL, scrape online, browse chapters, and optionally download data.
  - Steps:
    - [x] 3.1 Add new workspace crate + dependencies.
    - [x] 3.2 Build input-driven TUI screen for URL submission.
    - [x] 3.3 Fetch story data using resolver and display metadata + chapter list.
    - [x] 3.4 Add chapter reader view (open selected chapter, load content online when needed).
    - [x] 3.5 Add download/export action from TUI.
    - [x] 3.6 Run `cargo check` and fix compile issues.
  - Owner: assistant
  - Priority: high
  - Due: ASAP
  - Link: local implementation complete

## Backlog
- [ ] Add JSON schema version field for stable downstream contract.
- [ ] Add incremental sync manifest (`last_synced`, chapter hashes).
- [ ] Add retry/backoff/circuit-breaker to network proxy layer.

## Completed
- [x] 2026-02-16 — Implemented unified URL resolver + `nexus-tui` crate end-to-end.