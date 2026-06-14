# Velumia sprint ceremony (ChatPRD-first)

**Applies to:** every V1 Feature sprint (`/sprint-start LIE-NNN`)  
**Last update:** 2026-06-14

## Principle

ChatPRD is the **authoring surface** for sprint planning artifacts. Local files under `.ai/velumia-sprints/LIE-NNN/` are **mirrors** synced via **velumia-planning-chatprd-sync**. Both documents are **linked on the Linear issue**.

## Two documents per sprint

| Document | Owner | Timing | ChatPRD template |
|----------|-------|--------|------------------|
| Sprint PRD | PO | **Before** refinement | (PO structure) |
| Implementation Spec | Devs (SM coordinates) | **After** PRD agreement | **ChatPRD: Feature Implementation Spec** |

## Ceremony flow

```mermaid
flowchart TD
  SM[SM: sprint folder from templates]
  PRDCreate["PO: create sprint PRD in ChatPRD"]
  LinearLink1[Link Sprint PRD on Linear]
  SyncDown1[Sync sprint-prd.md]
  Refine["PO + Devs: refinement using PRD"]
  PRDUpdate["PO: update sprint PRD in ChatPRD"]
  SyncDown2[Re-sync sprint-prd.md]
  Agree{PO + Devs agree?}
  ImplCreate["Devs: create Implementation Spec in ChatPRD"]
  LinearLink2[Link Implementation Spec on Linear]
  SyncImpl[Sync implementation-plan.md]
  Gate[Planning gate → In Progress]

  SM --> PRDCreate --> LinearLink1 --> SyncDown1 --> Refine
  Refine --> PRDUpdate --> SyncDown2 --> Agree
  Agree -->|yes| ImplCreate --> LinearLink2 --> SyncImpl --> Gate
  Agree -->|no| Refine
```

## Planning gate

- [ ] Sprint PRD created before refinement; updated after refinement; synced locally
- [ ] Implementation Spec created after PRD agreement; synced locally
- [ ] Both documents linked on Linear issue
- [ ] Implementation Spec includes sub-agent ownership and handoffs
- [ ] Story points on Linear issue
- [ ] Security Planning review complete
- [ ] ≤5 refinement rounds or stakeholder sign-off on escalations

## Implementation

SM delegates per **Implementation Spec** sub-agent ownership. Handoffs must complete before downstream subtasks close.

## Skills and agents

- Skill: `.cursor/skills/velumia-sprint-start/SKILL.md`
- Skill: `.cursor/skills/velumia-planning-chatprd-sync/SKILL.md`
- SM: `.cursor/agents/velumia-scrum-sm.md`
- PO: `.cursor/agents/velumia-scrum-po.md`

## ChatPRD project

`projectId: asst_WVuIAcqzH1O6ERmhWHE91UGL`
