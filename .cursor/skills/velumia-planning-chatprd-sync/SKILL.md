---
name: velumia-planning-chatprd-sync
description: Sync a sprint implementation PRD from ChatPRD MCP to local sprint folder, velumia-pm ChatPRD mirror, and Basic Memory. Use after create_document during Planning or when refreshing sprint PRD from ChatPRD.
---

# Velumia Planning — ChatPRD sync

Syncs a published sprint PRD from ChatPRD (Velumia project) to dual local storage and Basic Memory.

## Prerequisites

- ChatPRD MCP (`user-chatprd`) enabled
- Sprint folder exists: `.ai/velumia-sprints/LIE-NNN/`
- `documentUuid` recorded in `planning.md` (from `create_document` response)

## Velumia project binding

| Field | Value |
|-------|-------|
| Browser slug (UI only) | `1775405894874-velumia` |
| MCP `projectId` / `openaiAssistantId` | `asst_WVuIAcqzH1O6ERmhWHE91UGL` |

Pass **`projectId: "asst_WVuIAcqzH1O6ERmhWHE91UGL"`** on `create_document`. Verify via `get_document` → `createdInThread.assistant.name === "Velumia"` or `list_project_documents` with same `projectId`.

## Workflow

1. **Fetch document:** `get_document` with sprint `documentUuid`.
2. **Save raw import** (velumia-pm):
   - `velumia-pm/content/chatprd-velumia/_import/meta/<uuid>.json` — full JSON response (minus body if split)
   - `velumia-pm/content/chatprd-velumia/_import/body/<uuid>.md` — markdown `content` only  
   Use `python3 velumia-pm/scripts/materialize_chatprd_body.py` on stdin if piping JSON.
3. **Working copy** (velumia sprint folder):
   - Path: `.ai/velumia-sprints/LIE-NNN/sprint-prd.md`
   - YAML frontmatter: `chatprd_document_uuid`, `linear_issue`, `chatprd_url`, `synced_at`, `title`
   - Body: document content from ChatPRD
4. **Git mirror** (velumia-pm):
   - Path: `velumia-pm/content/chatprd-velumia/documents/implementation/lie-NNN-<slug>.md`
   - Use `python3 velumia-pm/scripts/render_chatprd_mirror.py` with JSON from `get_document`, or merge meta + body manually
   - Follow frontmatter conventions in `content/chatprd-velumia/README.md`
5. **Basic Memory:**
   - Upsert note **`Velumia — Sprint PRD — LIE-NNN`** under `projects/velumia/`
   - Include: goal summary, scenario IDs, `chatprd_url`, local paths, `synced_at`
   - Append row to **`Velumia — Artifacts`** index (issue id, uuid, mirror path)
6. **Optional:** Linear comment with ChatPRD doc link on Planning gate pass.

## Stakeholder escalation

If `get_document` content contains open questions not resolved in repo specs, **do not** mark sync complete. Escalate via SM → stakeholder; log in `decisions.md` under **Stakeholder — ChatPRD open items**.

## Related

- ChatPRD prefs: `velumia-pm/.cursor/preferences/chatprd-velumia-project.md`
- PO publishes via `create_document` with `projectId: "asst_WVuIAcqzH1O6ERmhWHE91UGL"`
