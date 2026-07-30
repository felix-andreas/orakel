# Archive audit, day 8: assume a fourth bug

**2026-07-30, model claude-opus-5 effort xhigh.** Three silent partial-data bugs were found
on three consecutive days, and the worst had no code to fix. Today's brief was to assume a
fourth. **There was a fourth, and a fifth, and one of them is unrecoverable.**

Method, unchanged from `existence-is-not-completeness`: enumerate what Friday's scoring and
the 08-02 review will actually **read**, then verify each artifact exists in R2, is complete,
and **can be read back with the contents it claims** — not that a file exists.

## Answer up front

| # | finding | severity | status |
|---|---|---|---|
| 1 | **3 ledger rows on 2 markets resolved YES on 07-29 and are not in `resolutions.csv`** | **critical** — silently flatters the headline and mis-reads the completeness gate | found, handed to the CEO |
| 2 | **`closed_time` is `0` for all 74 closed legs, always has been** — Gamma's `closedTime` is not RFC3339 | **high** — Friday's "has UMA settled this?" field carries no information | **fixed** + selftest guard |
| 3 | **`data/out/predictions_2026-07-26.csv` is in no archive and no container — permanently lost** | medium, unrecoverable | recorded; loss bounded, see §3 |
| 4 | `r2data verify` returns FAIL on a transient R2 HTTP 500 | medium — invites re-freezing over a good archive | documented in the runbook |
| 5 | the daily `tar` line is still hand-written | the day-6 root cause, unfixed until today | **fixed**: `scripts/freeze.sh` verifies its own contents |
| 6 | candle archive's apparent holes | **not a defect** — every one is a session-calendar artifact | verified, see §6 |

## 1. Two markets resolved YES on 07-29 and never got appended (critical)

`scripts/resolve_sweep.py`, run against all 61 outstanding markets:

| market | rows | outcome | Gamma `closedTime` | independent gate-0 check |
|---|---:|---|---|---|
| `will-wti-reach-85-in-july-2026-from-july-27` | 1 | **Yes** | 2026-07-29 16:32:12Z | WTIU6 max **85.56** ≥ 85 since ws 07-27 16:27Z ✓ |
| `will-xauusd-dip-to-4000-by-july-27-2026` | 2 | **Yes** | 2026-07-29 16:10:11Z | XAUUSD min **3996.19** ≤ 4000 since ws 07-26 22:00Z ✓ |

Both confirmed two independent ways: Gamma (`closed=true` + `outcomePrices ["1","0"]` +
`umaResolutionStatus: resolved`, identity-asserted) **and** our own frozen candles.

Why this is the critical one, not the tidy one:

- **It biases the headline in our favour.** Both are near-money legs that went **against**
  us. `reach-85` was emitted 07-29 at `q = 0.6176` against a market at 0.5950 — the model was
  *closer*, so that row is fine — but `dip-to-4000` carries 2 rows and is exactly the
  `dip-to` down-barrier family that the day-7 tail analysis identified as the variant's whole
  risk. Leaving them out of the resolved set removes two tail draws from the sample.
- **It mis-reads the completeness gate.** `ops/state.toml` says the review reads the evidence
  only when every outstanding row's market appears in `resolutions.csv`. These 3 rows would
  have made the gate read *unmet* on Friday for a bookkeeping reason rather than a UMA one,
  and the gate's only power is to delay the review by a day. **Append before judging.**
- **Nothing errored.** They resolved mid-week, between daily runs, on boards whose *other*
  legs are still open. No process was watching for a leg that settles early.

**The generalisation, which is new:** every completeness check we had asked "is the archive
complete **as of the last run**". None asked "**did something resolve while we were not
looking**". A market can leave the outstanding set without any run touching it.
`resolve_sweep.py` now asks that question, and should be run daily, not only on Friday.

## 2. `closed_time` has been 0 for every resolved leg, forever (fixed)

The fourth silent-data bug, and it is the same shape as the first three.

```
Gamma  endDate    : "2026-08-01T03:59:59.999Z"   <- strict RFC3339
Gamma  closedTime : "2026-07-29 16:10:11+00"     <- space separator, 2-digit offset
```

`parse_iso` was `DateTime::parse_from_rfc3339(s).ok()`, and the call site was
`.unwrap_or(0)`. The second format fails to parse, the error is swallowed, and the column
comes out `0`. Measured before the fix: **74 of 74 closed legs had `closed_time = 0`** — in
today's `legs.csv` and in every `legs.csv` this variant has ever written, including the two
backtest freezes.

- **Nothing computed a wrong number**, because `closed_time` is written, read back, and never
  used in any calculation. That is exactly why it survived eight days.
- **It was about to matter.** Friday's hardest question is "has this leg settled yet, and
  when" — and `closed_time` is the field a scorer under time pressure would reach for. Every
  leg would have looked like it settled at the Unix epoch.

**Fixed**: `parse_iso` now widens `+00` / `+0000` offsets and accepts a space separator, with
an assertion in `ladderrv selftest` covering all three shapes so it cannot regress silently.
`legs.csv` regenerated — 74/74 now parse. The pricer is untouched and every selftest pricing
number is byte-identical.

> The pattern across all four: **a field that is present, parses, and carries no
> information.** `p.exists()` meant "complete"; a `tar` line meant "the archive"; and
> `unwrap_or(0)` meant "unknown" while looking like "the epoch". The audit question is not
> "is the value there" but **"could this value be wrong in a way that produces no error?"**

## 3. `predictions_2026-07-26.csv` is gone (unrecoverable)

Enumerated every `predictions_*.csv` across all eight R2 archives, local disk, and git:

| date | where it lives |
|---|---|
| 07-23 | `backtest-2026-07-23`, `backtest-metals-2026-07-25`, `live-2026-07-24` |
| 07-24 | `backtest-metals-2026-07-25`, `live-2026-07-24` |
| 07-25 | `backtest-metals-2026-07-25`, `live-2026-07-25` |
| **07-26** | **nowhere — no archive, no container, not in git** |
| 07-27 | `live-2026-07-27`, `live-2026-07-29`, `live-2026-07-30` |
| 07-28 | `live-2026-07-29`, `live-2026-07-30` (rescue freeze) |
| 07-29 | `live-2026-07-29`, `live-2026-07-30` |
| 07-30 | `live-2026-07-30` |

Day 4 cut no `live-*` freeze at all (the 07-27 manifest note says as much about candles: *"day
4 skipped the freeze entirely"*). By the time the 07-29 rescue freeze ran, the 07-26 file was
already off disk, so the rescue caught 07-27/28/29 and could not catch 07-26.

**What is lost, precisely** — stated narrowly so it is not over-claimed:

- Lost: the per-leg book snapshot for day 4 — `bid`/`ask`/`tob$` and the model `q` for the
  ~80 legs that were **suppressed**. Day 4's suppression counts can never be re-audited.
- **Not lost:** the 13 rows day 4 actually emitted (`results/proposed-rows-2026-07-26.csv`
  and the ledger, both in git), `predictions/fills.csv`, the candles, and the resolutions.
- **No Friday number and no 08-02 number depends on it.** The RV/IV comparison could never
  have used it (07-26 predates the IV columns by two days); the pricer split reads
  `pricer_version` from the ledger in git.

So: a real, permanent hole in the record, with a bounded blast radius. Recorded rather than
papered over.

## 4. `r2data verify` FAILs on a transient 500 (documented)

`verify` on the 11 manifests returned `FAIL data/candles-2026-07-25.tar.gz.r2.json: HEAD ...
returned unexpected HTTP 500`. The object is **fine** — `r2data pull` fetched it and verified
its sha256 (18,382,731 bytes) minutes later.

This is a nastier failure mode than it looks, on a day when someone is in a hurry: a FAIL on
the resolution record invites re-freezing, and re-freezing after 21:00Z over an archive you
believe is broken is how a good archive gets replaced by a worse one. **A FAIL must be
retried and confirmed with `pull` before it is believed.** In the runbook.

## 5. The hand-written `tar` line is now a script that checks itself (fixed)

The day-6 root cause was never fixed — only that day's damage was. The duty was still "cut
both freezes", executed by typing a `tar` line every morning. Finding 3 is the same failure
recurring one day earlier than the one we already knew about.

`scripts/freeze.sh` now holds the required-contents manifest **in git**, builds both
archives, and **re-reads each tarball it just built**, failing if any promised entry is
missing. It also counts sub-60-byte JSON entries, which is the shape a Pyth `no_data` stub
takes. `tape/` and `clob*/` were added to the live freeze — both are gitignored, `tape/` is
the only evidence behind a tape-gate suppression, and `clob60/` is what `cmd_analyze` reads
for every checkpoint Brier.

Cut and **read back out of R2** today, not merely verified:

- `candles-2026-07-30` — 912 entries, 19.4 MB. Read back: `candles/WTIU6/2026-07-29_a.json`
  parses `s=ok`, **1379 candles** (the 07-29 stub in yesterday's archive was correctly
  refetched to a full session by the `complete_through` rule).
- `live-2026-07-30` — 64 entries, 4.4 MB: `out/predictions_{07-27,07-28,07-29,07-30}.csv`,
  `legs.csv`, 9 `events_live/`, 48 `tape/`. Read back: today's predictions file has 83 legs
  and columns 15–18 are `sigma_rv,sigma_iv,q_iv,q_blend`.

## 6. The candle archive's holes are all session-calendar artifacts (not a defect)

Reported as a result, because "we looked and found nothing" is worth as much as a finding and
the previous three days make the null result non-obvious. Every day-file in
`candles-2026-07-29` from 07-20 on was opened, parsed and counted:

| apparent hole | explanation | verdict |
|---|---|---|
| all Pyth keys, 07-25 = 0 candles | Saturday; WTI/metals feed shut Fri 21:00Z → Sun 22:00Z | correct |
| WTI/metals 07-26 = ~120 | Sunday 22:00Z open → midnight = 120 min | correct |
| SPY/NVDA 07-25, 07-26 = 0 | weekend, no RTH | correct |
| WTI/metals 07-24 = ~1261 | Friday session ends 21:00Z → 1260 min in the calendar day | correct |
| everything 07-29 = 74–75 | the freeze ran at 01:29Z on 07-29; the day was 74 min old | correct, and refetched today |
| SPY/NVDA 07-29 = 52-byte `no_data` | at 01:29Z, 07-29's RTH had not happened | correct, and refetched today |
| WTIV6 07-21 = 1359 vs 1382 | 23 one-minute gaps in the deferred contract | genuine thin-feed gaps; CLU6 is the active month for July, so no July answer depends on it |

**Local state after today's fetch:** 07-29 complete on every key (WTIU6 1379, SPY/NVDA 390
RTH candles, 00:00Z–23:59Z), 07-30 partial as expected at 01:13Z.

The one standing exposure this leaves is stated in the runbook §1: **if no run happens after
07-31 21:00Z, the resolution record for 07-31 is a ~74-minute stub** and gate 0 for the whole
batch is unanswerable. That is the single most load-bearing action of Friday.

## What Friday needs that no freeze covers

Unchanged from 07-29, all now scripted rather than described:

1. Fresh `discover` — `legs.csv`'s `closed`/`winner`/`volume` are a snapshot. Done today
   (207 legs, 12 boards); redo Friday. **Now also carries a real `closed_time`.**
2. Fresh `clob data 60` **after** 21:00Z — it refetches correctly since 07-29.
3. The both-ways, identity-asserted `condition_ids` lookup — `scripts/resolve_sweep.py`.
4. Candles force-refetched after 21:00Z, WTIU6 **and** WTIV6, then `scripts/freeze.sh`.
5. **New: `resolve_sweep.py` daily, not just Friday** — see §1. A market can resolve while
   nobody is looking at it.
