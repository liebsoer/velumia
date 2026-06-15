# Shared libraries (future)

Velumia Nx monorepo shared code lives under:

| Folder | Purpose |
|--------|---------|
| `libs/ui/*` | Vue components, composables, theme tokens, shared TS types for the UI |
| `libs/desktop/*` | IPC contract types, Tauri helpers, shared Rust-adjacent TS utilities |

**This sprint:** no packages created — code remains in `apps/ui` and `apps/desktop` until a Feature Implementation Spec declares extraction into a lib.
