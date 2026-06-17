---
name: velumia-planning-chatprd-sync
description: Sync sprint PRD or Implementation Spec from ChatPRD MCP to local sprint folder, velumia-pm mirror, and Basic Memory. Use after create_document or update_document during Planning.
---

# Velumia Planning — ChatPRD sync

Pulls authoritative ChatPRD documents into local mirrors. ChatPRD is **never** updated by this skill — use `create_document` / `update_document` first, then sync.

## Prerequisites

- ChatPRD MCP (`user-chatprd`) enabled
- Sprint folder exists: `.ai/velumia-sprints/LIE-NNN/`
- `documentUuid` recorded in `planning.md` (separate UUIDs for sprint PRD and Implementation Spec)

## Velumia project binding

| Field | Value |
|-------|-------|
| Browser slug (UI only) | `1775405894874-velumia` |
| MCP `projectId` / `openaiAssistantId` | `asst_WVuIAcqzH1O6ERmhWHE91UGL` |

Pass **`projectId: "asst_WVuIAcqzH1O6ERmhWHE91UGL"`** on `create_document`. Verify via `get_document` → Velumia project context.

## Document types

| `document_type` | ChatPRD title pattern | Local mirror | velumia-pm mirror |
|-----------------|----------------------|--------------|-------------------|
| `sprint-prd` | `Velumia — Sprint PRD — LIE-NNN — …` | `.ai/velumia-sprints/LIE-NNN/sprint-prd.md` | `velumia-pm/content/chatprd-velumia/documents/implementation/lie-NNN-<slug>-sprint-prd.md` |
| `implementation-spec` | `Velumia — Implementation Spec — LIE-NNN — …` | `.ai/velumia-sprints/LIE-NNN/implementation-plan.md` | `velumia-pm/content/chatprd-velumia/documents/implementation/lie-NNN-<slug>-impl-spec.md` |

## Linear links (before or after sync)

Link each ChatPRD document on the sprint issue via Linear MCP `save_issue`:

```yaml
id: LIE-NNN
links:
  - url: <chatprd_url>
    title: Sprint PRD          # or "Implementation Spec"
```

Append-only; safe to call again when adding the second link.

## Workflow

1. **Fetch document:** `get_document` with the relevant `documentUuid`.
2. **Save raw import** (velumia-pm):
   - `velumia-pm/content/chatprd-velumia/_import/meta/<uuid>.json`
   - `velumia-pm/content/chatprd-velumia/_import/body/<uuid>.md`  
   Use `python3 velumia-pm/scripts/materialize_chatprd_body.py` on stdin if piping JSON.
3. **Working copy** (velumia sprint folder) — path per table above:
   - YAML frontmatter: `chatprd_document_uuid`, `linear_issue`, `chatprd_url`, `synced_at`, `title`, `document_type`
   - Body: document content from ChatPRD
4. **Git mirror** (velumia-pm): path per table above; follow frontmatter in `content/chatprd-velumia/README.md`
5. **Basic Memory:**
   - Sprint PRD → note **`Velumia — Sprint PRD — LIE-NNN`**
   - Implementation Spec → note **`Velumia — Implementation Spec — LIE-NNN`**
   - Append row to **`Velumia — Artifacts`** (issue id, uuid, document type, mirror path)
6. Update `planning.md` sync flags (`Synced locally: yes`).

## When to run

| Event | Action |
|-------|--------|
| PO creates sprint PRD (`create_document`) | Sync `sprint-prd`; link on Linear |
| PO updates sprint PRD after refinement (`update_document`) | Sync `sprint-prd` again |
| Devs create Implementation Spec (`create_document`) | Sync `implementation-spec`; link on Linear |
| Devs update Implementation Spec during sprint (`update_document`) | Sync `implementation-spec` again |

## Stakeholder escalation

If document content has open questions not resolved in repo specs, **do not** mark sync complete. Escalate via SM → stakeholder; log in `decisions.md` under **Stakeholder — ChatPRD open items**.

## Related

- Ceremony: `.cursor/skills/velumia-sprint-start/SKILL.md`
- Nx layout (mirrored Implementation Spec paths): `.cursor/skills/velumia-nx-monorepo/SKILL.md`
- ChatPRD prefs: `velumia-pm/.cursor/preferences/chatprd-velumia-project.md`
- Implementation Spec template (mandatory): [`templates/chatprd/chatprd_feature-implementation-spec.tpl.md`](../../templates/chatprd/chatprd_feature-implementation-spec.tpl.md) — ChatPRD name **ChatPRD: Feature Implementation Spec**. Local mirror: `.ai/velumia-sprints/LIE-NNN/implementation-plan.md` must follow the same section structure after sync.
