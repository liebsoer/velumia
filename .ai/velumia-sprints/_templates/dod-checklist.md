# DoD checklist — LIE-NNN

**QA:** · **SM:** · **Date:**

Copy from [`_templates/dod-checklist.md`](../_templates/dod-checklist.md) at sprint start — do not reuse a stale checklist from a prior sprint. If `_templates/dod-checklist.md` changes mid-sprint, reconcile this file before Review.

## Product / acceptance

- [ ] 1. All BDD scenario IDs green in CI
- [ ] 2. Linear acceptance criteria met
- [ ] 3. No scope creep bundled into Done

## Engineering

- [ ] 4. PR merged with LIE-NNN (feature branch `lie-NNN-*`, not direct to `main`)
- [ ] 5. `authorize()` on new protected actions
- [ ] 6. LangDock from Tauri backend (if applicable)
- [ ] 7. Migrations applied and tested (if this sprint adds or changes migrations under `apps/desktop/migrations/`)

## Quality / security / architecture

- [ ] 8. QA sign-off
- [ ] 9. Security review complete (`security-review.md` § Implementation; no open Critical/High without waiver)
- [ ] 10. Architecture review complete (`architecture-review.md` § Implementation; no open Critical/High without waiver)
- [ ] 11. No P0/P1 on issue scenarios

## Process

- [ ] 12. Sprint folder complete
- [ ] 13. Stakeholder Review accepted
- [ ] 14. Retro captured (`retro.md` with Category + Tooling sign-off when applicable)
- [ ] 15. BM Status updated
