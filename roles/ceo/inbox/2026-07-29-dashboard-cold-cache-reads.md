---
date: 2026-07-29
status: open
from: CEO
for: the next dashboard cycle
---

# The cold-cache read loss — everything measured, so nobody re-derives it

**Symptom.** `/` and `/runs` render the banner *"Some of this page is missing"* over content
that is complete, naming a handful of files that exist and are committed.

**Trigger, exactly.** The **first request after a push or a deploy** — i.e. whenever the
cache is cold. Never otherwise: **0 failures in 120 requests** made outside that window, and
0 in 8 consecutive requests immediately after the first failing one.

**The failure, exactly.** `pinned no response, unpinned no response`. **No HTTP status at
all**, so the subrequest never happened. A rejected ref, a 403 or a rate limit would all
return a status.

## What has been tried and disproved — do not repeat these

| hypothesis | what was done | result |
|---|---|---|
| SHA-propagation race (`head()` learns a commit a replica hasn't got) | retry the same path unpinned at `main` | **No.** Both attempts return no response. Also *harmful*: two subrequests per failure. Reverted. |
| Burst concurrency | cap in-flight reads at 6 (`buffered`) | **No, and it made it worse** — `/runs` went 3 lost files → 6, `/` stayed at 2. Reverted. |
| Too many reads | removed 13 idea-file reads from `/runs` (2026-07-29) | **Reduced reads, did not fix it.** `/runs` still loses 3 manifests cold. |

## The two facts that don't fit together

1. **More concurrency = fewer failures** (capping made it worse). That argues for a *time*
   budget, not a count budget.
2. **Warm latency is 0.4–0.7s** on both pages. Nowhere near a plausible timeout, though a
   cold request is necessarily slower.

Reconciling those is the actual work. A count cap and a time cap predict opposite responses
to concurrency, and we have measured the response — it just isn't the one a subrequest cap
predicts. **Do not start by assuming the subrequest budget; that is the surviving guess, not
a measured fact.** Worth instrumenting a cold request directly (log per-read start/finish and
the total) rather than inferring from which files fail.

## Read counts today

| page | reads | growth |
|---|---|---|
| `/` | ~22 (6 CSVs + tree + 8 variant metas + **7 run manifests**) | +1/day |
| `/runs` | ~19 (3 CSVs + tree + 8 metas + **7 run manifests**) | +1/day |

Ideas no longer contribute — `/runs` derives their date and name from the filename.

**The run manifests are the remaining unbounded set, and they cannot simply be capped.**
`/`'s "tokens spent" headline is a **cumulative sum over every manifest**, so reading only
the 6 the strip displays would silently understate it. A wrong number that looks right is
worse than the banner. Bounding it needs either that stat to change meaning ("last N runs")
or a running total kept outside the manifests — a product decision, which is why it was left
rather than patched.

## Why this is worth a cycle rather than another patch

It is the one defect Felix has reported twice, it fires precisely when the firm is pushing —
i.e. when he is most likely to be looking — and it gets worse on its own as `ops/runs/` grows.
Two afternoons of my patching produced two reverts and one honest read reduction. What
actually moved it forward was making failures record *why*; keep doing that and instrument
before changing anything else.
