---
linear_issue: LIE-NNN
document_type: implementation-spec
chatprd_document_uuid:
chatprd_url:
synced_at:
title:
chatprd_template: "ChatPRD: Feature Implementation Spec"
chatprd_template_file: templates/chatprd/chatprd_feature-implementation-spec.tpl.md
---

# Implementation Spec — LIE-NNN (local mirror)

> **Do not author here first.** Devs create this document in ChatPRD via `create_document` **after** PO + devs agree on the sprint PRD.
>
> **Mandatory template:** [`templates/chatprd/chatprd_feature-implementation-spec.tpl.md`](../../../templates/chatprd/chatprd_feature-implementation-spec.tpl.md) (ChatPRD title: **ChatPRD: Feature Implementation Spec**). Copy section structure from that file; do not use a custom outline.
>
> This file is synced from ChatPRD by **velumia-planning-chatprd-sync**.

**Linear:**  
**ChatPRD:** (url)

---

# ChatPRD: Feature Implementation Spec

## Section 1 – Feature Summary

What this feature does in 1-2 sentences, why it matters, and who uses it.

### Scope

- **In scope**:
- **Out of scope**:
- **Prerequisites**:
- **Depends on**:

## Section 2 -Acceptance Criteria

### Must-Have (P0)

1.

### Should-Have (P1)

2.

### Nice-to-Have (P2)

3.

## Section 3 - User Flow

### Happy Path

1. **Entry point**:
2. **Action**:
3. **Feedback**:
4. **Result**:
5. **Next step**:

### Alternate Paths

- **Path B**:
- **Path C**:

## Section 4Edge Cases & Error States

### Input Validation

-

### System Errors

-

### Permission & State

-

## Section 5 - Technical Approach

### Files to Create/Modify

Per subtask, declare **lib placement**: `libs/ui/*`, `libs/desktop/*`, or **inline-in-app**.

| # | Subtask | Sub-agent | Lib placement | Files | Done when | Status |
|---|---------|-----------|---------------|-------|-----------|--------|
| 1 | | | inline-in-app / libs/ui/* / libs/desktop/* | | | ⬜ |

### Sub-agent ownership

| Sub-agent | Subtasks |
|-----------|----------|
| velumia-dev-backend | |
| velumia-dev-frontend | |
| velumia-dev-bdd | |
| velumia-dev-devops | |
| velumia-dev-langdock | |
| velumia-dev-security | (review only) |

### Handoffs

| From | To | Deliverable | Done when |
|------|-----|-------------|-----------|
| | | | |

### Dependency order

1.

### Data Flow

1.

### New Dependencies

-

## Section 6 -

Dependencies & Risks

### Dependencies

- **Upstream**:
- **Downstream**:
- **External**:

### Technical Risks

-

### Rollout Strategy

-

## Section 7 - Test Plan

### Manual Testing

1.

### Automated Tests

- **Unit tests**:
- **Integration tests**:
- **E2E tests**:

### Regression Checks

-
