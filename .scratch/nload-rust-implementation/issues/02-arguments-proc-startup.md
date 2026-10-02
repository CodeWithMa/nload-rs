# Implement arguments, proc snapshots and startup ordering

Status: open
Labels: wayfinder:task
Type: task
Mode: AFK
Parent: [Implement and verify the minimal Rust port of nload](../map.md)
Assignee:
Blocked by: 01

## Question

Can the Arguments and Proc sampler modules supply validated invocation outcomes, complete original-byte counter snapshots and deterministic startup selection under specification sections 2, 3.1, 4.1 and 8?

Implement the strict `args_os()` grammar, independent ceiling validation/conversion, Cargo-authoritative help/version outcomes, raw-byte identity, unbounded natural-digit comparison and selection validation. Implement fresh whole-file proc acquisition through EOF and complete checked snapshot parsing, preserving the distinction between global failure and absence in a valid snapshot. Keep terminal/clock work out of argument parsing and counter acquisition timestamps under the application/clock ownership specified by the handoff.

Provide meaningful deterministic coverage for applicable A01–A09, S01–S02 and raw-name/lifecycle acquisition cases. Initial startup control-flow/no-effect assertions are completed by the integration ticket, not assumed from parser tests. Document the caller-facing configuration and snapshot interfaces for the monitor/application consumers, and update the case-to-evidence record with precise coverage and remaining integration gaps. Resolution records implemented interfaces and passing appropriate locked tests.
