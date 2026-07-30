# Friday 2026-07-31 — scoring runbook for barrier-touch/ladder-rv

**Written 2026-07-30 (day 8), model claude-opus-5 effort xhigh — the last run before the
evidence freezes.** Companion to `friday-2026-07-31-readiness.md` (what resolves) and
`archive-audit-2026-07-30.md` (what is and is not in the archive).

> Assume you are under time pressure and will not re-derive any of the reasoning. Run the
> commands in order. Every trap below has already cost this project a wrong number once.
> **Nothing here requires a judgement call except the two places that say STOP.**

---

## 0. Timeline — there are TWO freeze moments, not one

| when | what closes | what you must do after it |
|---|---|---|
| Fri **20:00Z** | SPY / NVDA week-of-Jul-27 (RTH close) | equity candles for 07-31 become complete |
| Fri **21:00Z** | **WTI + gold + silver** July monthlies **and** week-of-Jul-27 weeklies — 104-ish rows | §1–§6 (the main pass) |
| Sat **04:00Z** | BTC July monthly (11:59pm ET) — 16 rows | repeat §2, §3, §6 for the BTC rows |

Do **not** start §2 before 21:00Z. Gamma will happily return open markets and you will
record 59 "still open" rows and think the batch failed.

**`endDate` is not the resolution window.** Gamma reports the July monthlies as
`2026-08-01T03:59:59.999Z`. The window ends at the last session close, **07-31 21:00Z**.
Never score a monthly off `endDate`.

---

## 1. Freeze the resolution record FIRST, before any scoring

The candle archive is the only thing that can ever answer "did it touch". Pyth deletes
expired contract feeds; if this is skipped, gate 0 for July is gone forever.

```bash
cd strategies/barrier-touch/ladder-rv
cargo build --release
./target/release/ladderrv selftest data          # must print `ok` on every line incl. parse_iso

# candles for yesterday+today, ALL NINE keys. WTIU6 *and* WTIV6.
for K in BTCUSDT ETHUSDT USOILSPOT SPY NVDA WTIU6 WTIV6 XAUUSD XAGUSD; do
  ./target/release/ladderrv candles data "$K" 2026-07-30 2026-07-31
done
./target/release/ladderrv vol data
```

The refetch of an incomplete day is automatic since 07-28 (`complete_through`: a day-file is
re-pulled unless it was written after that day ended). **Confirm it actually happened** —
this is the check, not the fetch:

```bash
python3 - <<'EOF'
import json,glob,os
for key in ['WTIU6','WTIV6','XAUUSD','XAGUSD','SPY','NVDA','USOILSPOT']:
    n=0
    for p in glob.glob(f'data/candles/{key}/2026-07-31*.json'):
        d=json.load(open(p)); n+= len(d.get('t',[])) if isinstance(d,dict) else len(d)
    print(f"{key:<10}{n:>6} candles on 2026-07-31")
EOF
```

**Expected after 21:00Z:** WTI/metals keys **≈1260** (00:00–21:00Z), SPY/NVDA **≈390** (RTH).
**STOP if any WTI/metals key is under 1200 or either equity key is under 380** — you are
holding a truncated session and every gate-0 answer off it is wrong. Re-run that key's
`candles` call and re-check. A 52-byte file whose JSON says `{"s":"ok"}` with an empty `t`
is the shape this bug takes; it is not an error.

Then freeze **both** archives — the script verifies its own tar contents, which is the fix
for the day-6 failure where `out/` was frozen nowhere:

```bash
bash scripts/freeze.sh 2026-07-31
```

Repeat this whole section after Sat 04:00Z with `2026-07-31 2026-08-01` and
`freeze.sh 2026-08-01`, for the BTC legs.

> A `verify` FAIL is **not** proof the archive is lost. R2 returned a transient HTTP 500 on
> a HEAD for `candles-2026-07-25` on 2026-07-30 and the object was perfectly fine —
> confirmed by `r2data pull`. **Retry a FAIL, and confirm with `pull`, before concluding
> anything.** Never re-freeze over a good archive on the strength of one FAIL.

---

## 2. Resolve every outstanding row — union both query forms, assert identity

**Do not hand-write Gamma queries.** The three traps are in the script:

```bash
python3 scripts/resolve_sweep.py                          # report
python3 scripts/resolve_sweep.py --emit /tmp/res-0731.csv # + candidate resolutions rows
```

What it does and why (all three have bitten us):

1. **`closed` is a FILTER whose default is `false`, not an override.**
   `?condition_ids=<cid>` returns **open** markets and `[]` for closed ones;
   `&closed=true` is the exact reverse. **No single query finds both.** On Friday the batch
   is *mixed* — boards close at 21:00Z but UMA settles over hours — so a one-form scorer
   drops half the batch **silently**, because `[]` is a valid 200. The script unions both.
2. **`?condition_id=` singular is not an error.** Gamma ignores the unknown parameter and
   returns an arbitrary unrelated market with a 200 (it once returned "New Rihanna Album
   before GTA VI?" for a WTI condition id). The script therefore **asserts
   `returned.conditionId == the cid asked for`, per market**, and refuses to emit a
   resolution on any mismatch. It also asserts the slug matches the ledger's.
3. **`closed` with non-final `outcomePrices` is UNSETTLED, not a 50/50.** A leg can be
   closed while UMA has not adjudicated; `outcomePrices` is then absent or `["0.5","0.5"]`.
   The script classifies that as unsettled and never converts it to an outcome.

The script prints three buckets. Act on them like this:

| bucket | what it means | what to do |
|---|---|---|
| **SETTLED** | closed + `outcomePrices` final + identity checked | hand the `--emit` rows to the CEO for `predictions/resolutions.csv` |
| **UNSETTLED** | still open, or closed with UMA lagging | **re-run the script**, do not guess. See §3 |
| **NEEDS A HUMAN** | identity mismatch, slug drift, or repeated network failure | **STOP.** Do not average around it. One wrong conditionId poisons a whole asset's Brier |

**`predictions/` has exactly one writer — the CEO.** Never append. Emit to a scratch path
and hand it over.

---

## 3. When a leg has not settled yet

This is the normal state at 21:05Z, not a failure. UMA settles a batch over hours.

1. **Wait and re-run `resolve_sweep.py`.** Cheap and idempotent. Poll every ~20 min.
2. **Do not substitute a price for an outcome.** A last mid of 0.98 is not a YES. The
   completeness gate is about *resolutions*, not prices.
3. **Do not fill it from our own candles.** Gate 0 is 756/760, not 760/760, and the four
   misses are exactly the near-barrier cases — `2/7 feed-misses inside 0.10%` resolved YES
   anyway (`venue-resolution-epsilon`). Our mirror is evidence *about* the venue, never a
   substitute for it. Compute the gate-0 answer, report any disagreement loudly, and let the
   venue's answer be the outcome.
4. **The completeness gate (`ops/state.toml`) may only DELAY the review, once.** Its exact
   words: the review reads the evidence only when every outstanding row's market appears in
   `predictions/resolutions.csv`; if not, the review slips **one day and no further**. It
   may **never** be used to reopen a completed review. If the batch is still incomplete on
   08-02 after one slip, that is a decision to argue in `ops/decisions.md`, not an automatic
   second slip.
5. **Three rows are already settled and were sitting outside the gate** — see §6, item 1.
   Append those before judging whether the gate is met, or the gate reads as failed for a
   bookkeeping reason rather than a UMA reason.

---

## 4. Regenerate the derived evidence, in this order

```bash
# price history: this GROWS until close, and until 2026-07-29 it was cached on mere
# existence -- a stale clob60 would have scored the trial against prices stopping 07-25,
# dropping legs one at a time via price_at's max-age guard, with no error at all.
./target/release/ladderrv discover data "<all 12 board slugs>"   # refresh closed/winner/volume
./target/release/ladderrv clob data 60
./target/release/ladderrv analyze data
```

The 12 board slugs (copy exactly — one comma-separated argument, **never** space-separated;
space-separated silently prices only the first board):

```
what-price-will-wti-hit-in-july-2026,what-price-will-wti-hit-in-august-2026,what-price-will-xauusd-hit-in-july-2026,what-price-will-xagusd-hit-in-july-2026,what-price-will-bitcoin-hit-in-july-2026,will-wti-hit-week-of-july-27-2026,will-xauusd-hit-week-of-july-27-2026,will-xagusd-hit-week-of-july-27-2026,will-spy-hit-week-of-july-27-2026,will-nvda-hit-week-of-july-27-2026,will-spy-hit-week-of-july-20-2026,will-nvda-hit-week-of-july-20-2026
```

Then `fills.csv` — **a Brier headline without it is a calibration result and must be called
one** (`midpoint-is-not-a-fill`). Expect the split by board family to matter: reachable
fraction of the scored midpoint is **WTI 99% / silver 89% / gold 82% / SPY-NVDA weekly 38%**.

---

## 5. The numbers to report — exactly these, with these labels

### 5a. Completeness, first and unconditionally

- outstanding rows / markets, and **settled and appended / total**. As of 2026-07-30 the
  ledger plus today's 5 proposed rows gives **131 outstanding rows over 62 markets**.

### 5b. Headline calibration

- Paired Brier, **model minus market midpoint**, at the **daily in-window checkpoint** —
  never window-open, which does not survive the leg-sum gate for gold.
- **Per asset and pooled**, each with **`Σmid` vs `Σwinner`** beside it. This family is
  **nested**, so a literal `leg-sum ≈ 1` gate cannot fail and must not be quoted
  (`checkpoint-artifact`). Prior values: 1.38 at creation, 1.11 window-open, 1.28 daily.
- Each with its **fillable count** and `exec_edge`.
- Gate board-snapshots at `avg_mid ≤ 0.40` before quoting any Brier margin.

### 5c. The pricer split — report it as INCONCLUSIVE, in both units

Settled in advance in `ops/decisions.md` (2026-07-29). **Do not re-open it, and do not go
looking for a unit or a subset that clears.** Measured from the ledger on 2026-07-30,
including today's 5 rows:

| arm, within `feed_open = 1` | rows | distinct markets |
|---|---:|---:|
| `ladder-rv/2026-07-23-touch-prob` | 48 | 36 |
| `ladder-rv/2026-07-27-touch-prob-jump` | **40** | **19** |
| `feed_open = 0` (its own line, **never pooled**) | 43 | 33 |

- **Clears n ≥ 30 in rows. Does not clear it in markets, and cannot by Friday.**
- **Markets is the honest unit** — the readiness doc's item 7 said so before any of this,
  and 13 of the 19 jump-arm markets carry more than one row.
- **So: report the split INCONCLUSIVE.** The row number goes beside it as descriptive only.
  **The 08-02 decision may not rest on this split.**
- Today's run confirmed the exhaustion prediction rather than fixing the power: 5 rows and
  **exactly 1 new market**.

### 5d. The RV/IV pre-registration — scored, not switched

`prereg-rv-iv-blend-2026-07-28.md`. Decision rule fixed before the outcome; do not touch it.

- **Primary anchor = the emission time (~01:1xZ)**, which is what the recorded `q`s carry.
  **12:00Z is a robustness check only.** Fixed blind in `ops/decisions.md`.
- **Scorable from the 07-29 and 07-30 files only** — 07-28's file predates the `q_iv` /
  `q_blend` columns. Do **not** re-derive them; that is a change to the comparison by the
  person scoring it.
- Power, measured 2026-07-30: **67 legs (07-29) + 64 legs (07-30), union 68 distinct legs**,
  63 in both days. Against a floor of n ≥ 30 the **leg** count clears comfortably, and it
  clears in the market unit too — unlike §5c. **Its real limitation is two days and one
  regime, not small n.** Say it that way.
- **Tradeability veto baseline, recorded now, blind:** sell signals on fundable
  (`mid ∈ [3c,50c]`, `feed_open=1`) legs were **A_rv 4 / B_iv 1 / C_blend 1** on 07-29 and
  **A_rv 2 / B_iv 1 / C_blend 1** on 07-30. **B and C both reduce sell signals on both
  scorable days**, so both fail veto item 3 before the outcome is known. The prereg's
  "everything passes" branch is therefore almost certainly unreachable, and the expected
  recorded conclusion is *"IV/blend may be better calibrated than RV and is not usable by a
  sell-only variant."*
- **A correction to our own claim, measured 2026-07-30.** Day 7 reported "OVX (57.1) has
  fallen below RV14 (62.7) for the first time", and `ops/decisions.md` carries it as a
  softening of the prereg's premise. **It compared OVX to the wrong series.** The pricer's
  effective σ is the *intraday* realized vol, not RV14. Measured from the frozen files:

  | day | asset | σ_rv (bumped, **in use**) | σ_iv | |
  |---|---|---:|---:|---|
  | 07-29 | WTI | 0.5133 | 0.5773 | IV above |
  | 07-30 | WTI | 0.5261 | 0.6793 | IV above |
  | both | gold, silver | 0.18–0.19 / 0.38–0.39 | 0.2468 / 0.4783–0.4868 | IV above |

  **IV sat above the σ actually used on every asset on both scorable days — the premise
  never softened.** (OVX itself went 57.15 → **67.59** on 07-29, with VIX 18.21 → 20.66 and
  VXSLV printing a 54.10 high: a real cross-asset vol event, not a data artifact.) This
  changes no rule and re-specifies nothing; it corrects a fact we had recorded wrongly.

### 5e. What Friday cannot tell us — put this in the review, not a footnote

The trial has **one regime**: WTI 90.46 (Fri 07-24 close) → 83.68 Sunday open → 77.80 low
Tuesday → 83.86 now. Every outstanding WTI leg is priced against that round trip. A good
Friday is evidence the model handles *this* selloff-and-bounce, not that it handles a
market. **104 rows are not 104 independent trials** — see §5f.

### 5f. Sizing, which is what 08-02 actually turns on

Full derivation in `results/sizing-2026-08-02-prep.md`. The three numbers to carry:

1. **The pooled sell-side break-even bound clears on nominal n and fails on effective n.**
   `q* = 0.822`, `q = 0.868`; `q⁻ = 0.829` at nominal n=356 (**clears by +0.7pp**) and
   `q⁻ = 0.808` at effective n=173 (**fails by −1.3pp**).
2. **Effective n is 173, not 356.** Intraclass correlation of the loss indicator within a
   monotone family (same board, same direction) is **ρ = 0.325** at a mean 4.24 legs per
   family → design effect 2.05.
3. **The exposure is a cliff, not a tail.** The outstanding WTI down-ladder collected
   **1.15** of premium across 21 rows. At the realised low (77.80, −14.0%) it loses 0.66 and
   is still net **+0.49**. Another 3.7% down, to 75, and it loses **6.96** — net **−5.81**.
   The marginal cost of the next 5% leg down is **+6.30 = 548% of that family's entire
   premium.**

---

## 6. Three things to do before anything else on Friday

1. **Append the three already-settled rows.** Found 2026-07-30 by
   `resolve_sweep.py`; both resolved YES on 07-29 and neither is in `resolutions.csv`:

   | market | rows | outcome | Gamma `closedTime` | gate 0 |
   |---|---:|---|---|---|
   | `will-wti-reach-85-in-july-2026-from-july-27` | 1 | **Yes** | 2026-07-29 16:32:12Z | WTIU6 max 85.56 ≥ 85 ✓ |
   | `will-xauusd-dip-to-4000-by-july-27-2026` | 2 | **Yes** | 2026-07-29 16:10:11Z | XAUUSD min 3996.19 ≤ 4000 ✓ |

   Both are `dip-to`/near-money legs that went against us, so leaving them out flatters the
   headline. Append them **before** judging the completeness gate.

2. **Rebuild the binary and run `selftest`.** `parse_iso` was fixed on 07-30 (Gamma's
   `closedTime` is `2026-07-29 16:10:11+00` — space separator, 2-digit offset, **not
   RFC3339**, so `closed_time` was silently `0` for all 74 closed legs in every `legs.csv`
   ever written). The selftest now asserts all three formats. **If it fails, stop** — a
   `legs.csv` with `closed_time = 0` will make every leg look like it settled at the epoch.

3. **Re-read `ops/decisions.md`.** The pricer split, the RV/IV anchor, and the refutation of
   the "structurally short downside touch" hypothesis are all **settled**. 08-02 is a
   **sizing** question. Do not spend Friday re-litigating any of the three.

---

## Appendix — every silent wrong-answer path found so far, as a checklist

| # | trap | how it shows up | guard |
|---|---|---|---|
| 1 | `ladderrv live` with space-separated slugs | prices only the **first** board, writes a complete-looking file | one comma-separated arg |
| 2 | `cmd_live` overwrites `predictions_<date>.csv` per call | last run wins | all boards in one call |
| 3 | `candles`/`clob`/`tape` cached on `p.exists()` | truncated series, legs leave the scored set one at a time | `complete_through`, fixed |
| 4 | daily freeze's hand-written `tar` line | `out/` frozen nowhere; `predictions_2026-07-26.csv` **lost forever** | `scripts/freeze.sh`, verifies its own contents |
| 5 | `r2data verify` HEADs the blob | "verified" while holding a 21.9KB-of-69.7KB file | read contents back; `pull` + inspect |
| 6 | `r2data verify` transient HTTP 500 | a good archive reads as FAIL | retry, confirm with `pull` |
| 7 | `?condition_ids=` without/with `closed=true` | half the mixed batch silently dropped | union both, §2 |
| 8 | `?condition_id=` singular | 200 with an arbitrary unrelated market | assert identity, §2 |
| 9 | `closedTime` not RFC3339 | `closed_time = 0` for every resolved leg | `parse_iso` fixed + selftest |
| 10 | `endDate` on a monthly | 2026-08-01T03:59Z, not the 07-31 21:00Z window | never score off `endDate` |
| 11 | `closed` + non-final `outcomePrices` | reads as a 50/50 outcome | classified unsettled, §2 |
| 12 | literal `leg-sum ≈ 1` on a nested ladder | gate **cannot fail**, gets recorded as passed | report `Σmid` vs `Σwinner` |
| 13 | checkpoint at board creation | 85% of legs quote ~0.50; a flat base rate beats the market | daily in-window anchor |
| 14 | selecting an entry at the **last** qualifying checkpoint | look-ahead: a touching leg's mid rises toward 1 first | first qualifying checkpoint |
