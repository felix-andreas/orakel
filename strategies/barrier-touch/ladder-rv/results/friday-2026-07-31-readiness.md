# 2026-07-31 21:00Z — what state Friday's scoring has to be in

Written 2026-07-28 (day 6), model claude-opus-5 effort xhigh, three days out. This is the
trial's real evidence: the 08-02 review turns on it.

## What resolves, exactly

**120 outstanding rows over 58 markets. Identity 58/58 clean** (verified today by
`condition_ids` lookup, token id checked against `clobTokenIds`).

| resolves | rows | markets | what |
|---|---:|---:|---|
| **Fri 2026-07-31 21:00Z** | **104** | 42 | WTI + gold + silver July monthlies **and** the week-of-Jul-27 weeklies |
| Sat 2026-08-01 04:00Z | 16 | 16 | BTC July monthly (crypto, 11:59pm ET) |

Gamma reports the monthlies' `endDate` as `2026-08-01T03:59:59.999Z`; the **resolution
window** still ends at the last session close, **07-31 21:00Z**. Do not score the monthlies
off the `endDate` field.

## The lookup trap that will silently eat rows on Friday

`GET /markets?condition_ids=<cid>` — **`closed` is a FILTER whose default is `false`, not an
override.** Measured today on live and resolved markets:

| query | open market | closed market |
|---|---|---|
| `?condition_ids=<cid>` | **returns it** | `[]` |
| `?condition_ids=<cid>&closed=true` | `[]` | **returns it** |
| `?condition_ids=<cid>&closed=false` | **returns it** | `[]` |

**There is no single query that finds a market in both states.** Friday's scorer must try
both and take whichever returns a row. This matters precisely on Friday, because at 21:00Z
the boards close but UMA resolution lags — so the set will be *mixed*, some closed and some
still open, and a scorer using one form drops the other half **silently**, since `[]` is a
valid 200 response. (Memory's earlier note — "`condition_ids` returns [] for closed markets,
`&closed=true` fixes it" — was half the rule; the other half breaks open markets.)

Related: **`?condition_id=` (singular) is not an error.** Gamma ignores the unknown
parameter and returns an arbitrary unrelated market — today it returned "New Rihanna Album
before GTA VI?" for a WTI condition id. A typo returns a wrong answer with a 200.

## The split by pricer version — and the confound that must be respected

The ledger now carries `pricer_version`. Outstanding rows:

| pricer_version | feed_open=1 | feed_open=0 | total |
|---|---:|---:|---:|
| `ladder-rv/2026-07-23-touch-prob` | 50 | **45** | 95 |
| `ladder-rv/2026-07-27-touch-prob-jump` | **25** | 0 | 25 |

**Every stale-feed row is also an old-pricer row.** On the `feed_open=0` side the two
factors are perfectly confounded and no split can separate "the stale feed hurt us" from
"the old pricer was wrong". Therefore:

> **Score the pricer comparison only within `feed_open = 1`: 50 `touch-prob` rows against
> 25 `touch-prob-jump` rows.** Report the 45 shut rows separately, as their own line, and
> never pool them into a pricer conclusion.

**The jump arm is underpowered at 25 rows** against the ≥30 floor. Wednesday's and
Thursday's runs are the only chance to fix that — at ~14 rows/run it reaches ~50 by Friday.
**If either run is skipped, the pricer split cannot be decided on Friday**, and that is the
main operational risk between now and then.

## Checklist for Friday

1. **Runs happen Wed 07-29 and Thu 07-30.** Not optional — see above.
2. **Backfill applied** to the 132 pre-existing rows from
   `results/ledger-backfill-2026-07-28.csv` (measured, not estimated). Without it 95 rows
   aggregate as `unversioned` and the split has nothing to split.
3. **Candle archive frozen to R2 on 07-31 after 21:00Z**, force-refetching 07-31 itself —
   that archive is the resolution record, and the fix committed today makes the refetch
   automatic. WTIU6 **and** WTIV6.
4. **Both-ways market lookup** in whatever scores the boards (§ above).
5. **`fills.csv` regenerated after resolution**, so every Brier number carries its fillable
   count and `exec_edge`. Per `midpoint-is-not-a-fill`, a Brier headline without it is a
   calibration result and must be called one. Expect the split by board family to matter:
   WTI 99% / silver 89% / gold 82% reachable.
6. **`Σmid` vs `Σwinner` reported beside the headline** — this family is nested, so a
   literal leg-sum is vacuous (`wiki/reference/checkpoint-artifact.md`).
7. **Repeated daily rows on the same market are one correlated observation, not six.** 120
   rows sit on 58 markets. The per-market number is the honest one; the CEO owns this
   aggregation.
8. **The RV/IV comparison is scored, not switched** —
   `results/prereg-rv-iv-blend-2026-07-28.md`, decision rule fixed before the outcome.

## What Friday cannot tell us

The trial has one regime: WTI fell from 90.46 (Fri 07-24 close) to 81.84 (07-28), roughly
−9.5% in two sessions, and every WTI leg outstanding is priced against that move. A good
Friday is evidence that the model handles *this* selloff, not that it handles a market. Say
so in the review rather than letting 104 rows read as 104 independent trials.
