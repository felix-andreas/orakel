---
date: 2026-07-29
status: open  # instrumented 2026-08-02; concurrency refuted by measurement
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


---

## 2026-08-02 — instrumented, and one more hypothesis refuted

The standing instruction here was *instrument a cold request before changing anything else*.
Done. Every read now counts itself and the numbers ride in an HTML comment in the page footer
(`reads: attempted=… hit=… net=… failed=… span_ms=…`), in the response rather than a log —
a cold request creates a new isolate, so a counter queried afterwards belongs to a different one.

**Reproducible on demand:** push a commit, then poll until the HEAD memo rolls over to the new
SHA. Two independent runs gave the same numbers to the read.

| request | attempted | cache hits | network | failed | span |
|---|---|---|---|---|---|
| `/runs` cold | 35 | 0 | 35 | **22** | 369 ms |
| `/runs` warm | 24 | 21 | 3 | 0 | 130 ms |
| `/state` cold | 2 | 0 | 2 | 0 | 113 ms |

**Concurrency is refuted, by measurement rather than by argument.** Bounding in-flight reads to
4 gave `attempted=35 hit=0 net=35 failed=22` — identical — and cost 163 ms. The 07-28 note
claiming it "made things worse" was an artifact of counting distinct file names in a banner
rather than counting reads; that note is corrected in `data.rs`. Time is refuted too: 369 ms.

**The surviving hypothesis, with arithmetic.** The per-request subrequest budget, *and the Cache
API spends it too*. A cold read costs ~3 subrequests (`cache.get` miss → `fetch` → `cache.put`)
where a warm read costs 1. So `/runs` costs ~24 warm and ~70 cold against a 50 ceiling — which
predicts it runs out around read 16, i.e. 13 successes and the rest failing. That is what we
measured. It also explains why pacing cannot help (a count, not a rate), why `/state` never
fails, and why only a SHA change triggers it.

**Next experiment, decisive and cheap:** disable the Cache API for one deploy. If the theory
holds, cold `/runs` drops to ~24 subrequests and `failed` goes to 0. If `failed` stays at 22,
the theory is wrong and the remaining candidate is the read count alone. Written up in
`dashboard/src/live.rs` next to the code it concerns.
