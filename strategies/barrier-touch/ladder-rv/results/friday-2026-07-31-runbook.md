# Friday 2026-07-31 — scoring runbook for barrier-touch/ladder-rv

**Written 2026-07-30 (day 8), model claude-opus-5 effort xhigh — the last run before the
evidence freezes.** Companion to `friday-2026-07-31-readiness.md` (what resolves) and
`archive-audit-2026-07-30.md` (what is and is not in the archive).

> Assume you are under time pressure and will not re-derive any of the reasoning. Run the
> commands in order. Every trap below has already cost this project a wrong number once.
> **Nothing here requires a judgement call except the two places that say STOP.**

> ### DRY-RUN LOG — 2026-07-31 01:1x–02:0xZ, claude-opus-5 effort xhigh
>
> Every command below was executed today against live data. **Five things were wrong and are
> corrected in place; each correction is marked `[DRY-RUN FIX]`.** What was *verified working*:
> `cargo build` + `selftest` (exit 0), the candle refetch rule (07-30's 73-minute stubs were
> repaired to 1381/1382/392 — this is the single most load-bearing mechanism here and it
> works), `resolve_sweep.py` (60 markets / 128 rows, 0 needing a human, exit 0), `vol`,
> `clob 60`, `analyze`, `freeze.sh` (both archives cut, pushed, **and read back out of R2**),
> and `scoring` (the `ci_lo`/`ci_hi` columns gate 1 depends on are present in `scores.csv`).
>
> **Two steps could not be tested before the close and are marked `[UNTESTABLE TODAY]`** with
> what to check instead.
>
> `git pull --rebase` **fails** in this checkout with "Cannot rebase onto multiple branches".
> Use `git pull --rebase origin main`.

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

**Every command in this document assumes you are in `strategies/barrier-touch/ladder-rv/`.**
The python checks use relative `data/...` paths and silently report zeros from anywhere else.

```bash
cd strategies/barrier-touch/ladder-rv
cargo build --release                            # ~10s
./target/release/ladderrv selftest data

# candles for yesterday+today, ALL NINE keys. WTIU6 *and* WTIV6.
for K in BTCUSDT ETHUSDT USOILSPOT SPY NVDA WTIU6 WTIV6 XAUUSD XAGUSD; do
  ./target/release/ladderrv candles data "$K" 2026-07-30 2026-07-31
done
./target/release/ladderrv vol data
```

> **`[DRY-RUN FIX]` The selftest does NOT print `ok` on every line.** It prints 15 lines and
> only 5 contain the word `ok`; the pricing-grid lines print a signed `diff` and no verdict.
> The pass criterion is **exit code 0** (`echo $?`), plus eyeballing that the three final
> `parse_iso(...) -> Some(1785341411) ok` lines are present. Scanning for "ok on every line"
> will make a healthy binary look broken at 21:05Z.

The refetch of an incomplete day is automatic since 07-28 (`complete_through`: a day-file is
re-pulled unless it was written after that day ended). **Confirm it actually happened** —
this is the check, not the fetch:

`[DRY-RUN FIX]` — the original block omitted **BTCUSDT/ETHUSDT**, which is exactly what the
Saturday pass needs. Use this version; it takes the date as an argument and knows what each
key is *supposed* to have on that weekday:

```bash
python3 - 2026-07-31 <<'EOF'
import json,glob,sys,datetime
day=sys.argv[1]; dow=datetime.date(*map(int,day.split('-'))).strftime('%a')
EXP={'WTIU6':1260,'WTIV6':1260,'XAUUSD':1260,'XAGUSD':1260,'USOILSPOT':1200,
     'SPY':380,'NVDA':380,'BTCUSDT':1400,'ETHUSDT':1400}
print(f"--- {day} ({dow}) ---")
for key,floor in EXP.items():
    n=0
    for p in glob.glob(f'data/candles/{key}/{day}*.json'):
        d=json.load(open(p)); n+= len(d.get('t',[])) if isinstance(d,dict) else len(d)
    print(f"  {key:<10}{n:>6}  floor {floor:>5}  {'OK' if n>=floor else '*** STOP ***'}")
EOF
```

**Verified reference counts (measured today from the archive, Friday 2026-07-24):**
WTIU6/XAUUSD/XAGUSD **1261**, USOILSPOT **1246**, SPY/NVDA **392**, BTCUSDT **1440**.
A normal *mid-week* day is 1381/1382 (the feed runs to 23:59Z); **Friday is ~1261 because the
session ends 21:00Z.** So ≈1260 is right for 07-31 and is *not* a truncation.

**STOP if any key is under its floor** — you are holding a truncated session and every gate-0
answer off it is wrong. Re-run that key's `candles` call and re-check. A 52-byte file whose
JSON says `{"s":"ok"}` with an empty `t` is the shape this bug takes; it is not an error.

> **`[DRY-RUN FIX]` The Saturday 04:00Z pass needs a different expectation, or it will fire a
> STOP on every line and mean nothing.** On Saturday the WTI/metals feed is shut all day, SPY
> and NVDA are shut, and **only BTCUSDT/ETHUSDT matter** (they are the only 24/7 keys and the
> only ones the 16 BTC rows resolve on). For the Saturday pass:
>
> - run the block above for **`2026-07-31`** and require **BTCUSDT ≥ 1400** (the full Friday),
> - run it again for **`2026-08-01`** and require **BTCUSDT ≥ 240** (00:00–04:00Z),
> - **ignore every WTI/metals/equity line on both** — zeros there are the session calendar,
>   not a fault. (Verified today: Sat 07-25 is 0 on all seven non-crypto keys and 1440 on BTC.)

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

> **`[UNTESTABLE TODAY] #2` — the only branch that matters cannot be exercised before 21:00Z.**
> Run today at 01:2xZ the script worked end to end and exited 0, but everything is still open,
> so it reported **SETTLED 0 / UNSETTLED 60 markets (128 rows) / NEEDS A HUMAN 0**. The
> SETTLED path, the `--emit` writer and the identity-mismatch branch therefore ran against
> **zero** rows today. The three query traps are all exercised on the UNSETTLED path (both
> query forms were issued for all 60 markets with no identity failure), so the lookup half is
> proven; **the emit half is not.**
>
> **Check on Friday, before trusting `--emit`:** the row count in the emitted CSV must equal
> the SETTLED market count in the report, and its header must be exactly
> `market_slug,condition_id,winning_outcome,resolved_date,note` — **verified today as
> byte-identical to `predictions/resolutions.csv`'s header.** If `--emit` writes 0 rows while
> the report shows SETTLED > 0, stop; do not hand over an empty file.

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

> ### `[DRY-RUN FIX]` — THE BIGGEST ONE. `discover` DOES NOT REFRESH ANYTHING.
>
> `cmd_discover` calls `fetch_all`, and `fetch_all` skips any job whose output file already
> **exists** (`src/main.rs`, `let skipped = jobs.iter().filter(|(_, p)| p.exists())`). It has
> **no** `complete_through` guard — unlike `candles`, `clob` and `tape`, which were all fixed.
> Running it as written today printed **`discover: fetched 0, cached 12`** and re-read
> yesterday's JSON. On Friday at 21:30Z that means **every July board still reads
> `closed: false` with no winner**, and `legs.csv` is a snapshot of Thursday.
>
> **This was not hypothetical today.** Clearing the cache and refetching changed `legs.csv`
> from **207 legs to 209** and surfaced two markets listed after yesterday's run:
> `will-wti-reach-85-in-july-2026-from-july-30` (new, open) and
> `will-bitcoin-reach-65k-in-july-2026-from-july-30` (new, already closed) — plus
> `will-bitcoin-reach-65k-in-july-2026-from-july-28` flipping `closed False → True` with
> `outcomePrices ["1","0"]`. **You must move the cache aside first:**

```bash
# [DRY-RUN FIX] discover is cached on file EXISTENCE. Clear it or it is a no-op.
mv data/events data/events-stale-$(date -u +%m%d) && mkdir data/events

# price history: this GROWS until close, and until 2026-07-29 it was cached on mere
# existence -- a stale clob60 would have scored the trial against prices stopping 07-25,
# dropping legs one at a time via price_at's max-age guard, with no error at all.
./target/release/ladderrv discover data "<all 12 board slugs>"   # refresh closed/winner/volume
./target/release/ladderrv clob data 60
./target/release/ladderrv analyze data
```

Confirm `discover` prints **`fetched 12, cached 0`**. If it says `cached 12`, the `mv` did not
happen and every number after this point is Thursday's.

> ### `[DRY-RUN FIX]` — A NEW SILENT NO-OP: `tape`/`wash` are SPACE-separated
>
> Appendix trap #1 says "one comma-separated argument, **never** space-separated". That is
> true for `discover` and `live` (`args[3].split(',')`) and **exactly backwards for `tape` and
> `wash`**, which take `args[3..]` — a list of separate arguments. Passing a comma-joined
> string to `tape` looks for one board literally named `"a,b,c"`, matches no leg, runs the
> loop zero times, and **exits 0 with no output whatsoever.**
>
> It cost a real row today: the comma form left the WTI and silver tape files at Thursday's
> vintage, and the stale tape read **0 taker trades within 5c** on
> `will-xagusd-dip-to-56-by-july-27-2026` — a tape-gate suppression. Refetched properly, the
> same leg had **4**, and it is in today's emission.

```bash
# tape/wash: SPACE-separated, several arguments. Not commas.
./target/release/ladderrv tape data what-price-will-wti-hit-in-july-2026 will-xauusd-hit-week-of-july-27-2026
```

`tape` prints one line per leg. **Silence means it matched nothing** — check the separator.

The 12 board slugs (copy exactly — one comma-separated argument, **never** space-separated;
space-separated silently prices only the first board):

```
what-price-will-wti-hit-in-july-2026,what-price-will-wti-hit-in-august-2026,what-price-will-xauusd-hit-in-july-2026,what-price-will-xagusd-hit-in-july-2026,what-price-will-bitcoin-hit-in-july-2026,will-wti-hit-week-of-july-27-2026,will-xauusd-hit-week-of-july-27-2026,will-xagusd-hit-week-of-july-27-2026,will-spy-hit-week-of-july-27-2026,will-nvda-hit-week-of-july-27-2026,will-spy-hit-week-of-july-20-2026,will-nvda-hit-week-of-july-20-2026
```

Then `fills.csv` — **a Brier headline without it is a calibration result and must be called
one** (`midpoint-is-not-a-fill`). Expect the split by board family to matter: reachable
fraction of the scored midpoint is **WTI 99% / silver 89% / gold 82% / SPY-NVDA weekly 38%**.

`[DRY-RUN FIX]` — the original said "then `fills.csv`" without naming the commands. They are
firm-level tools, not the variant's, and **the order is load-bearing** (`roles/ceo/PLAYBOOK.md`
step 8): `fillcheck` writes the file, `scoring` joins it, and running `scoring` alone silently
drops the tradeability column.

```bash
cd /home/user/orakel
./tools/fillcheck/target/release/fillcheck     # writes predictions/fills.csv (atomic since 07-30)
./scoring/target/release/scoring               # joins it; writes predictions/scores{,_detail}.csv
```

**Verified today:** both binaries are built and `scoring` runs clean. Gate 1's statistic is
**not in the printed table** — it is in `predictions/scores.csv`, columns `n_markets`,
`mean_improvement_market`, `ci_lo`, `ci_hi`. Read the CSV, not the terminal.

> **`[UNTESTABLE TODAY] #1`** — `fillcheck` writes into `predictions/`, which has a single
> writer. I did not run it. What I *did* verify is that the binary exists, that `scoring`
> consumes its output, and that `scoring`'s current run reports `tradeability: 15/35 (43%)`,
> so the join is live rather than silently absent. **Check on Friday:** `fillcheck` must exit 0
> and `predictions/fills.csv` must have *more* rows than before, never fewer — a truncated
> `fills.csv` from a crashed run once read as 38% tradeable against a true 43% and parsed
> cleanly (`existence-is-not-completeness`).

---

## 5. The numbers to report — exactly these, with these labels

### 5a. Completeness, first and unconditionally

- outstanding rows / markets, and **settled and appended / total**.
  **`[DRY-RUN FIX]` The 131/62 figure is superseded.** The CEO appended the three settled rows
  on 07-30 (see §6 item 1, now done), so `resolve_sweep.py` measured **128 outstanding rows
  over 60 markets** at 01:2xZ today. **Plus today's 4 proposed rows on 4 already-covered
  markets → 132 outstanding rows over 60 markets** go into Friday.
- **Today's 4 rows add rows, not markets, and not power.** They resolve tonight, so they land
  in the **0–1d** horizon bucket, which currently scores −0.0001…+0.0008 over 19–20 rows at
  **3/20 fillable**. They were emitted because the policy says to emit them, and for no other
  reason; **they should not be expected to move gate 1**, which is judged per market.

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
Tuesday → 83.86 Thursday → **82.83 Friday 01:21Z**. Every outstanding WTI leg is priced
against that round trip. A good Friday is evidence the model handles *this*
selloff-and-bounce, not that it handles a market. **104 rows are not 104 independent
trials** — see §5f.

### `[DRY-RUN FIX]` The pre-declared list of what Friday CANNOT settle

Written 2026-07-31 01:5xZ, **before any 07-31 resolution exists**, so that on Sunday this is a
list being *read* rather than limits being *discovered* after the numbers are in. Anything not
on this list is fair game; anything on it must be reported as underpowered regardless of which
way it comes out.

1. **The pricer split.** Already settled as INCONCLUSIVE in `ops/decisions.md` (07-29) and
   §5c. Clears n≥30 in rows, not in markets. **Unchanged by today: today's 4 rows bought
   0 new markets.** The 08-02 decision may not rest on it.
2. **Anything about equity.** SPY/NVDA were feed-shut on **every** emission this week
   (19 legs suppressed again today, 5.4h stale). We have never once emitted an equity row
   with an open feed on the daily cadence. Friday cannot say whether the model is good or bad
   on equity — only that this *schedule* cannot trade it.
3. **The 0–1d horizon bucket.** ~20 of the scored rows sit there at **3/20 fillable** and a
   per-market mean of +0.0008 with a CI of ±0.003. It is flat and almost entirely unfillable.
   Today's 4 rows land here too. **A number this bucket produces is not evidence about the
   strategy**, in either direction.
4. **Between-family ρ, re-estimated on Friday.** 12 families is far too few for a stable ICC
   (today's estimate used 84). Friday cannot check the ρ = 0.326 that gate 4 turns on. The
   fix is more *boards*, and the July universe is exhausted.
5. **The 35–50c band**, where the only clearing edge lives (+5.2pp, +30.9% RoLC, n=65). It was
   found by reading a table of bands. Friday cannot validate it, and applying it now would be
   fitting a filter mid-trial. It is a candidate pre-registration for the August cohort.
6. **Regime generality.** One selloff-and-bounce in one month. Even a clean pass is
   single-regime evidence, and the August cohort spans a CME roll the July boards never saw.
7. **Whether the tail is "unlucky".** With `n_eff` ∈ [118, 173] on 356 backtest legs, 132
   forward rows over ~12 families cannot distinguish an unlucky draw from the central
   tendency. **This is the question Sunday will most want answered and it is the one the
   numbers are least able to answer** — say so before looking.

**The corollary, stated in advance:** if Friday comes back inconclusive, that is the
*expected* outcome for items 1–7, not a surprise that justifies an extension. Per
`ops/decisions.md`, extension needs a **named new source of evidence**, and the only one that
exists is the August cohort — which is a new trial with a new pre-registration, not a
continuation.

### 5f. Sizing, which is what 08-02 actually turns on

**`[DRY-RUN FIX]` The three gaps this section left open were closed on 2026-07-31 —
`results/sizing-2026-08-02-close-2026-07-31.md`. Carry these two numbers above all others:**

1. **The edge is smaller than the spread.** The nominal margin is **+0.73pp**; the median
   half-spread on gate-passing legs is **1.00c**. Selling at the **bid** rather than the mid
   puts `q* = 0.8316` against `q⁻ = 0.8289` — **fails by −0.27pp at nominal n, before any
   correlation argument and at a zero fee.** Break-even half-spread is 0.73c; the median
   gate-passing book is 2.0c wide.
2. **`n_eff` ∈ [118, 173]**, and the bound fails across that entire range (−1.21pp to
   −2.65pp). Between-family ρ is now measured, not assumed. **The one clustering level that
   "clears" (asset, 7 clusters) has ρ = 0.000 because 7 clusters cannot identify an ICC — do
   not quote it.**
3. **Kelly at the 95% lower bound is negative** (−6.8% at `n_eff` 173, −14.9% at 118). That
   answers "at our size" **without a bankroll**, so the missing bankroll is no longer a reason
   the question is open.

The original three numbers, from `results/sizing-2026-08-02-prep.md`:

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

1. ~~**Append the three already-settled rows.**~~ **`[DRY-RUN FIX]` DONE — skip this.** Verified
   today: both markets are in `predictions/resolutions.csv` (lines 23–24) and `resolve_sweep.py`
   now reports 23 resolutions and 60 outstanding markets. **Do not spend Friday on it.**
   Kept below for the record only.

   A result worth carrying into Sunday: both went **against us** on P&L, and both *improved*
   the per-market statistic (`+0.0319` and `+0.0178`), moving the variant mean from −0.0127 at
   21 markets to **−0.0094 at 23**. **"Lost money" and "beat the market" are different signs on
   the same row** — gate 1 measures the second. Do not narrate a loss as a calibration failure.

   Found 2026-07-30 by `resolve_sweep.py`; both resolved YES on 07-29:

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
| **15** | **`discover` cached on `p.exists()`** | **`fetched 0, cached 12`; boards read `closed:false` forever, new relists invisible (207 vs 209 legs today)** | **`mv data/events` aside first; require `fetched 12`** |
| **16** | **comma-joined slugs passed to `tape`/`wash`** | **matches no board, loop runs zero times, exits 0 with NO output; the gate then reads a stale tape (cost a row today)** | **`tape`/`wash` are SPACE-separated; `discover`/`live` are ONE comma-separated arg** |
| **17** | **`selftest` judged by "ok on every line"** | **only 5 of 15 lines say `ok`; a healthy binary looks broken** | **judge by exit code 0** |
| **18** | **Saturday's candle check run with Friday's expectations** | **STOP fires on all 7 non-crypto keys (session calendar), and BTC — the only key that matters — is not checked at all** | **Saturday: BTCUSDT only, ≥1400 for 07-31 and ≥240 for 08-01** |
