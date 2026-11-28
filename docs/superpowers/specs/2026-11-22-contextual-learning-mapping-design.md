# Contextual Learning Mappings

## Goal

In Bridge mode, relate a selected Ferro or Ferro construct to the generated Rust when the current export provides an exact source map. Extend the existing mutability explanation to conditionals and loops without making claims from stale, approximate, or generated-only mappings.

## Design

- Keep the shared `learningContextAtSelection` API and existing exact-range/stale guards.
- Classify a mapped snippet only when it clearly represents a Rust `let` binding, `if` conditional, `while` repetition, or the Ferro generated counted-loop label.
- Use the existing concept catalog IDs (`mutability`, `conditionals`, `loops`).
- Render the selected source and mapped Rust snippet; use a concept-specific Portuguese explanation.
- Leave manual concept selection and all three learning mode catalogs unchanged.
- Keep concept inference in the frontend mapping helper; do not modify compiler semantics or code generation.

## Acceptance

- Exact current source maps can resolve mutability, conditionals, and loops.
- Stale, approximate, generated-only, unrelated, or out-of-range mappings return no context.
- Existing mutability detail and source snippets remain unchanged.
- The panel does not imply translation when no exact mapped construct exists.

## Risks

Generated code shape can evolve. Classification must fail closed and tests must cover recognized forms and unrelated snippets.
