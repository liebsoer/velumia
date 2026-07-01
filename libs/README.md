# Shared libraries (future)

Velumia Nx monorepo shared code lives under:

| Folder | Purpose |
|--------|---------|
| `libs/ui/run-panel` | Shared run/session streaming panel (`RunPanel`, `RunPanelApi`) |
| `libs/ui/*` | Other Vue components, composables, theme tokens, shared TS types for the UI |
| `libs/desktop/*` | IPC contract types, Tauri helpers, shared Rust-adjacent TS utilities |

**This sprint:** no packages created — code remains in `apps/ui` and `apps/desktop` until an Implementation Spec ([`templates/chatprd/chatprd_feature-implementation-spec.tpl.md`](../templates/chatprd/chatprd_feature-implementation-spec.tpl.md), Section 5) declares extraction into a lib.
