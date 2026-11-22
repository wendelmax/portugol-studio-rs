# Rust preview stale state

## Intent

Keep the last generated Rust visible while the current source is being translated again, clearly mark it stale when it no longer matches the active source, and keep previews isolated per editor tab.

## Approaches

- Keep old output in the shared DOM during debounce: smallest change, but switching tabs can show one file's translation under another file.
- Cache the artifact and source snapshot on each tab: slightly more state, but preserves provenance and prevents cross-file display. **Chosen** because it matches the existing tab model and state contract.

## Requirements

- A translation artifact contains the source snapshot, generated Rust and source maps that produced it.
- Starting an update must not delete the previous artifact.
- While an artifact's source differs from the active document, show an explicit stale label and disable copy/save/detach actions.
- Old source maps are not navigable while stale; they become available again only for a fresh artifact.
- A successful current result replaces only the active tab's artifact. A failed or partial result keeps the prior artifact labeled stale and shows the current diagnostic.
- An asynchronous response for another tab or a superseded source revision is discarded.
- No prior artifact means the current loading/error state remains truthful and contains no fabricated Rust output.

## Non-goals

- Persisting generated previews across application restarts.
- Executing generated Rust or changing translation semantics.
