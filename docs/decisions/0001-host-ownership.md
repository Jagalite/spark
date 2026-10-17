# ADR 0001: The application owns the application

- Status: Accepted for the F01–F06 implementation scope
- Date: 2026-10-13

## Context

Terminal applications use different event loops, state containers, backend choices, and data cadences. Attractive art is useful only if applications can embed it without adopting another application framework.

## Decision

Keep terminal initialization/restoration, events, focus, layout, state, data acquisition, and frame presentation in the host. The runtime and catalogue expose ordinary Rust values and Ratatui cell rendering. They do not own clocks, timers, threads, networking, terminal I/O, a global animation manager, or the host's `AppState`.

The catalogue depends on the runtime. Gallery and tooling may depend on both. Full Ratatui and backend/tool dependencies stay outside the production libraries unless a concrete library requirement is reviewed. Constructors are typed; a gallery-only list does not justify a public dynamic registry.

The synchronous host is an executable example, not the only supported architecture. Later optional scheduling helpers must remain pure advice that an existing host can ignore.

## Consequences

- Callers supply time, geometry, palette, and profile explicitly.
- A minimal downstream consumer is a useful dependency/API check.
- The host must draw underlying content and choose composition order.
- Data acquisition cannot be coupled to animation frame cadence.
- The library cannot promise to repaint itself or restore a terminal it never owned.

## Alternatives left out

A mandatory Tokio runtime, prescribed action enum, scheduler thread, component framework, and all-design registry would enlarge the integration contract without evidence from the first design.

## Verification

Inspect production dependency trees and source for runtime I/O. Compile a downstream host. Keep claims of successful verification in [compatibility](../compatibility.md), not in this architectural decision.
