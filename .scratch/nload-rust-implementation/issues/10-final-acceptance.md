# Audit final acceptance evidence and the implementation handoff

Status: open
Labels: wayfinder:task
Type: task
Mode: AFK
Parent: [Implement and verify the minimal Rust port of nload](../map.md)
Assignee:
Blocked by: 09

## Question

Does the finished application satisfy every specification requirement with complete passing evidence, and are its source, locked dependencies, license, README and verification records ready for the agreed direct-binary handoff?

Audit all sections 1–11 and each A/S/V/T/D acceptance case against exact test names, recorded PTY/emulator/live observations and artifact reviews. Inspect module ownership, scripted seams, unsafe confinement, actual target/features/licenses, version authority, GPL v2 text and exact inspiration credit. Check that upstream departures and supersessions are implemented consistently and no pending observation has been counted as passing.

On the final code, run and record `cargo fmt --check`, `cargo test --locked` and `cargo build --release --locked`; record implementation revision plus any uncommitted diff, exact compiler/target/environment and current evidence. If final changes invalidate prior behavioral evidence, repeat affected checks. `Cargo.lock` must be tracked and included in the eventual explicitly authorized commit; request commit authorization if the specification's committed-lockfile condition is still outstanding, without silently weakening it or committing on your own.

Graduate concrete coverage/implementation/environment gaps into child tickets and dependencies, keeping final acceptance open until they pass. Close the map only when all children are closed, no in-scope fog remains and all required evidence and delivery conditions are satisfied. The resolution links the implementation handoff and verification record and accurately states the final repository state.
