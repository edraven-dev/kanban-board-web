# CLAUDE.md

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:

- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:

- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:

- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:

- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:

```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

## 5. Aggregate Boundaries Are Non-Negotiable

**SQL must never cross an aggregate boundary. This rule does not bend.**

The backend has exactly two aggregate roots: **Project** and **Board** (Board owns its
Columns, which own their Cards). Each aggregate root has **one repository**, and a
repository's SQL may only read/write the tables of **its own** aggregate.

- **No cross-aggregate joins or reads.** A query in the Board aggregate's repository may
  not `SELECT`/`JOIN`/`FROM` the `projects` table (or any table outside the Board
  aggregate), and vice versa. Intra-aggregate joins (Board↔Columns↔Cards) are fine.
- **Reference other aggregates by id only.** When one aggregate needs data or a fact
  about another (e.g. "does this project exist?"), it must obtain it **through a system
  service / port** that represents the other aggregate's public interface — never by
  querying the other aggregate's tables directly. In this monolith that port stands in
  for what would be an API call across a service boundary.
- The test: if you deleted the other aggregate's tables, this repository's SQL would
  still compile and make sense. If it wouldn't, you've crossed the boundary — stop.

A cross-aggregate foreign key kept purely as a database safety net (e.g.
`boards.project_id → projects.id`) is allowed, but it is **not** a licence to read across
the boundary in application queries.

## 6. Comments

**Comment sparingly. Document public interfaces.**

- No verbose or narrating comments. A comment earns its place only when the code can't
  say it itself — a non-obvious *why*, a subtle invariant, a gotcha. Prefer one tight
  line over a paragraph.
- Never restate what the code already says. If a comment paraphrases the next line,
  delete it. Strip existing verbose comments as you touch code.
- **Document public boundaries when the contract isn't obvious.** A public type, trait,
  or function crossing a module/aggregate boundary gets a concise doc comment (`///`
  rustdoc, JSDoc/TSDoc) *only* when its contract isn't already clear from the signature.
  One line stating what/why, never how. Don't doc self-evident items.
