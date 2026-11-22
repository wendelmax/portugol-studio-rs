# Contextual learning from current Rust source maps

## Intent

When a student selects a source declaration whose current generated Rust mapping is available, the Learning inspector should explain the mutability represented by that declaration. Keep manual concept selection for all unmatched or unavailable states, and label the evidence used for contextual explanations.

## Design

- Derive context only from the active tab's Rust preview artifact when its source snapshot matches the current editor document and the artifact is not stale.
- Accept only exact, source-backed mappings that contain the caret or the whole selected range.
- Inspect only the corresponding generated Rust span. A binding declaration (`let` or `let mut`) maps to the existing `mutability` concept; never infer concepts by parsing Portugol text with a regex.
- Prefer the narrowest matching source and generated spans when mappings overlap.
- Show whether the binding is immutable (`let`) or mutable (`let mut`), and explain the difference according to whether the active source is classic Portugol or Portugol-RS.
- Keep the existing concept picker. A deliberate manual choice takes precedence until the learner changes the editor selection again.
- When a mapping is missing, approximate, generated-only, stale or for another source revision, show no contextual claim and retain the selected manual topic.

## Out of scope

- Ownership/borrowing claims derived from editor text.
- Changes to the compiler, source-map contract, translation semantics or runtime.
- Automatic exercise completion or progress inference.
