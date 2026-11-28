# Home page language menu

## Intent

Keep the Ferro Studio start page recognizable while making the new `.fer` and `.rs` entry points available without adding more permanent toolbar buttons. The classic Ferro action remains the default, as it does today.

## Goals

- Keep one prominent “Novo Arquivo” action that opens a classic `.por` program and retains Ctrl+N behavior.
- Put Ferro and Rust creation choices in a compact, keyboard-accessible menu attached to that action.
- Preserve their existing templates and language detection; selecting an option must open the matching language in the editor.
- Keep the rest of the home page and its open, folder, help, recent, examples, and footer actions unchanged.
- Close the menu after selection and return focus to its trigger when Escape is pressed.

## Non-goals

- Redesign the whole start page, change its learning mode, or replace the examples browser.
- Change Ctrl+N, the workspace “new file” dialog, file extensions, or save behavior.
- Add dependencies or introduce a new frontend framework.
- Claim native Tauri visual validation from a browser-only screenshot.

## Constraints

- Use the existing static HTML, TypeScript event wiring, CSS variables, and Tauri application.
- Keep the established Ferro Studio home layout and theme tokens.
- Use existing `.por`, `.fer`, and `.rs` templates and existing `newTab` behavior.
- Test the menu controller with real event callbacks and test accessible markup; validate screenshots in native CI when available.

## Approaches

1. Keep separate “Novo Ferro” and “Novo Rust” buttons. This is simplest, but the start toolbar becomes crowded and drifts from the original three-action layout.
2. Add a compact language menu to “Novo Arquivo” while keeping its click and Ctrl+N behavior pointed at classic Ferro. This preserves the familiar default and exposes the new languages in one predictable place. **Selected**, because it improves the start toolbar without hiding the classic workflow or changing architecture.
3. Ask for a language in a modal every time. This makes the choice explicit, but adds friction to the common classic path and changes Ctrl+N behavior.

## Interaction and implementation

The new-file group contains the existing classic action plus a native `<details>/<summary>` disclosure. The disclosure contains two buttons with explicit labels “Novo Ferro (.fer)” and “Novo Rust (.rs)”. The controller receives language-specific callbacks so it can close the menu after activation and close/refocus it on Escape. The two language templates remain in `main.ts`; no template generation moves into the view helper.

The CSS scopes large icon-button styling to direct children of the start toolbar. The menu items use the existing surface, border, text, and accent tokens, and appear above the example column without changing the two-column page layout. The disclosure summary has a stable accessible name and test id; menu actions retain stable ids.

## Verification

- Unit tests exercise choosing each alternate language, closing the menu, and Escape focus restoration.
- Markup tests verify the classic action remains primary and the two alternate choices are present and labeled.
- `npm test`, `npm run build`, `cargo fmt --all -- --check`, Rust workspace tests and Clippy validate the frontend and integration.
- Playwright can render the Vite page, but its browser-only environment lacks Tauri IPC and therefore is not valid evidence for native startup, examples loading, icons initialized by Tauri startup, or the final native screenshot. Linux/macOS/Windows native CI remains authoritative for that path.

## Rollback

Revert the menu markup/controller/styles and restore the two direct actions in `main.ts`. No persistent state, file format, IPC command, or migration is introduced.
