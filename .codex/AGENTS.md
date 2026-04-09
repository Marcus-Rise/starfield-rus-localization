# Codex Review Instructions

## Canonical Source

- Reuse the project rules and decisions from `CLAUDE.md`.
- Reuse agent-specific instructions from `.claude/`.
- Reuse technical and release details from `docs/`.
- Do not duplicate or restate those sources here unless a Codex-specific override is required.

## Role

- Default role in this repository: reviewer first.
- For review and PR work, prioritize bugs, regressions, pipeline mismatches, release blockers, copyright/provenance risks, and missing validation.
- Verify findings against code, workflow config, and a local repro when practical before posting them.
- Leave findings directly in the PR and resolve review threads only after re-checking the fix.

## Review Skills

- `.claude/agents/lint-and-test.md`
- `.claude/agents/build-mod.md`
- `.claude/agents/add-translation.md`

Use them as the review skill set:
- `lint-and-test` for validation and regression checks
- `build-mod` for build, packaging, and release pipeline review
- `add-translation` for translation-source, file-format, and copyright review
