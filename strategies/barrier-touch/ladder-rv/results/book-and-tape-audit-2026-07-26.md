# Book and tape audit — where our fills actually are

**Date:** 2026-07-26 · **Run by:** slot-1 researcher · **Model:** claude-opus-5 (effort xhigh)
**Method:** `tools/fillcheck`'s YES-equivalent folding, re-run forward from each row's own
timestamp over every market this variant has ever predicted on (70 markets, 108 rows).

> In plain English: yesterday's audit said only 2 of our first 21 predictions were at a
> price anyone would trade. That number was true and it was also mostly about one kind of
> market. Split the same measurement by board family and the picture separates cleanly:
> on the weekly share-price ladders the quoted price is worth 38 cents on the dollar; on
> the monthly oil board it is worth 99.

## 1. The fill question, asked of every board family

For each market, take our latest prediction row, then replay the public trade feed from
that instant. A taker who **sold** Yes at q proves a resting **bid** at q — the price a
seller of ours could have hit. Prices are folded into the row's own outcome units.

| family | markets | a bid ≥ our mid, ever | …within 1h | Σ(mid) | Σ(best bid, capped at mid) | **ratio** |
|---|---:|---:|---:|---:|---:|---:|
| **WTI monthly** | 16 | 7 | 2 | 2.547 | 2.516 | **99%** |
| BTC monthly | 16 | 10 | 0 | 1.100 | 1.097 | **100%** |
| Silver monthly | 10 | 1 | 0 | 0.473 | 0.422 | **89%** |
| Gold monthly | 11 | 0 | 0 | 0.328 | 0.268 | **82%** |
| **SPY/NVDA weekly** | 17 | 0 | 0 | 0.406 | 0.154 | **38%** |
| all | 70 | 18 | 2 | 4.853 | 4.456 | 92% |

The ratio is the number that matters: how much of the price we were scored against was
demonstrably reachable. Averaged per market the haircut is **0.19c on WTI, 1.48c on the
equity weeklies** — and the equity weeklies' mids average 2.4c, so the haircut there is
**62% of the price**.

Twenty-four-hour bid-side taker flow on the live WTI July legs, for scale: ↑95 **$27,703**,
↑100 $11,353, ↓85 $7,688, ↑105 $3,401, ↓80 $2,953. On the gold July board the same window
is $0–155 per leg; on silver $0–86.

**Conclusion.** The 2/21 headline was a fact about *equity weeklies and sub-3c wings*, not
about the variant. The commodity monthlies are a real market on the side we would take.
Gold is the caveat: its Brier edge is the best of any asset (day-3) and its book is the
thinnest of the three monthlies — 0/11 markets ever showed a bid at our mid, and its
top-of-book is $1–19. **Gold has earned the right to trade and still has nowhere to
trade.**

## 2. A tight quote is not liquidity either — the tape gate

The week-of-Jul-27 boards were re-read today (they quoted 0.020/0.980 placeholders when
last checked on 2026-07-25).

| board | state today |
|---|---|
| WTI week-of-Jul-27 | still dead. 12/14 legs quote 1c/99c-class spreads. The two exceptions are ↓60 (0.001/0.007) and ↑125 (0.002/0.069, 6.7c → fails the gate); ↓60's mid is 0.4c, far under the fundable floor. |
| Gold week-of-Jul-27 | dead. 13/14 legs at 0.01/0.99. |
| Silver week-of-Jul-27 | dead. **14/14** legs at 0.01/0.99. |
| SPY week-of-Jul-27 | alive but wide — every leg 11c–55c spread. All fail. |
| **NVDA week-of-Jul-27** | **quotes look genuinely alive**: ↓180 0.026/0.05, ↓184 0.03/0.07, ↓188 0.06/0.10, ↓196 0.24/0.25, ↑220 0.25/0.27, ↑232 0.02/0.07 — six legs passing the 5c spread gate, `liquidityNum` $470–780 each. |

The NVDA board is the interesting one, so we asked the tape:

| leg | quote | trades, lifetime | bid-side notional |
|---|---|---:|---:|
| ↓180 | 0.026 / 0.050 | **0** | $0 |
| ↓184 | 0.030 / 0.070 | **0** | $0 |
| ↓188 | 0.060 / 0.100 | **0** | $0 |
| ↓196 | 0.240 / 0.250 | **0** | $0 |
| ↑232 | 0.020 / 0.070 | **0** | $0 |
| ↑220 | 0.250 / 0.270 | 2 | $28 |

A whole ladder quoted 1–5c wide by a market maker, and **not one counterparty has ever
appeared**. This passes every gate the variant currently has. It is the third distinct
failure mode of "the quoted price":

1. `phantom-midpoints` — the book is empty and the quote is fabricated (0.02/0.98).
2. `midpoint-is-not-a-fill` — the book is alive and wide, and the midpoint is not the bid.
3. **this one** — the book is alive *and tight*, and there is no one on the other side at
   all. A spread test cannot see it; only the tape can.

**New gate adopted today (the tape gate):** before a leg is predicted, require
demonstrated taker flow on the side we would take, in the price band we would trade —
concretely, ≥ 1 trade in the last 7 days on the bid side within 5c of the quote. And the
spread gate becomes **relative**: `spread ≤ min(5c, ½·mid)`, because a 1.6c spread on a
0.3c/1.9c book (August ↓20) "passes" a 5c test while the midpoint is 3.8× the bid.

Consequence today: **no week-of-Jul-27 rows and no equity-weekly rows.** The NVDA board is
demoted-to-research-only anyway (CEO, 2026-07-25), and a research row whose board has
never traded teaches us about the market maker, not about the market.

## 3. Identity check before the 07-31 resolution

Every one of the 70 markets in our ledger re-resolved cleanly today: **slug → same
conditionId in 70/70, token_id → `clobTokenIds[0]` in 70/70. No slug drift, no token
drift.**

One live hazard found, and it is on our side of the fence:

> **`GET /markets?condition_ids=<id>` returns `[]` for a closed market**, exactly like
> `?slug=`. All 18 lookups that "failed" in this check were the 18 already-resolved
> markets; adding `&closed=true` fixes every one.

`wiki/recipes/polymarket-api.md` documents this gotcha for `?slug=` only. A scorer that
looks markets up by `condition_id` without `closed=true` will silently fail to find
precisely the resolved markets it needs — which is the 07-31 batch. Flagged to the CEO.

Other resolution notes for 07-31:

- `will-wti-dip-to-90-in-july-2026` (our one genuinely liquid scored row) has **resolved
  YES**; we said 0.8263 against a 0.82 mid, so the side was right.
- A **re-added** ↓90 leg now exists — `will-wti-dip-to-90-in-july-2026-from-july-25`,
  listed 2026-07-25 16:2xZ, quoting 0.933/0.961. Its window opens at Monday's session
  open; do not confuse it with the resolved one when scoring. Board titles keep lying;
  four other July legs carry `-from-july-20`/`-from-july-25` private window starts.
- Resolution-epsilon screen, recomputed per leg from its **own** window start against our
  WTIU6 archive: nothing is inside 0.2%. Closest are ↓85 (window from 2026-07-22 16:28Z,
  running low 86.122, 1.32% clear), ↑95 (running high 93.488, 1.59% clear) and ↓80
  (window from 2026-07-20 16:30Z, running low **81.397**, 1.75% clear). ↓80 is the one to
  watch: its window low is $1.40 above the barrier with five sessions left.
- The July board's own mid-month CLQ6 → CLU6 roll does not change any live leg — see
  `august-roll-model-2026-07-26.md` §5.

## 4. Today's signals and the 13 proposed rows

Inputs @ 2026-07-24 21:00Z (last print; the market has been shut all weekend):
WTI CLU6 **90.46**, RV14 **48.8%**, OVX 68.0 · Gold **4053.31**, RV14 **20.4%**, GVZ 24.3 ·
Silver **58.20**, RV14 **41.3%**, VXSLV 48.1. Remaining window = **5 sessions**
(Jul 27–31), τ = 0.019841 yr.

**Two tier-A sell signals, both WTI, both with real depth:**

| leg | book | mid | q_rv | q_iv | edge | bid-side depth | 24h bid flow |
|---|---|---:|---:|---:|---:|---:|---:|
| ↓75 (from Jul 20) | 0.090 / 0.110 | 0.100 | 0.0064 | 0.0504 | **−9.4c** | $334 | $643 |
| ↓80 (re-touch) | 0.400 / 0.410 | 0.405 | 0.0738 | 0.1995 | **−33.1c** | $248 | $2,953 |

Both clear the 5c spread gate, the $100 depth bar, the resolution-epsilon screen and the
tape gate. Neither is a wing lottery: ↓80 is a 40c leg. Priced at the **bid** (0.400, the
only price we could actually hit), selling it receives 40c and locks 60c of collateral to
2026-07-31; expected P&L is `0.400 − 0.0738 = +32.6c/share` on 60c locked over 6 calendar
days — **54% on locked capital, ~3,300% annualised** if the model is right, which is the
triple `execution/DESIGN.md` §3 asks for rather than a cents-per-trade number. The venue
fee is `shares × 0.04 × 0.400 × 0.600` = **0.96c/share** on entry (`finance_prices_fees`,
taker-only), and nothing at resolution because redemption is not a match. ↓75 at the bid
(0.090) is +8.4c on 91c locked, ~9% over the hold — the same trade with four times the
capital tied up per cent earned, which is exactly the distinction cents/trade hides.

**No gold signal, no silver signal.** Every fundable metals leg sits within ~2c of the
model (gold ↓3900 q 0.179 vs 0.168; ↑4300 q 0.040 vs 0.034; silver ↓52 q 0.053 vs 0.066).
That is the third consecutive day gold has been tradeable-in-principle and silent.

**The one large disagreement is a buy, so we cannot take it.** `will-wti-reach-95` quotes
0.195/0.237 (mid 0.216) where the model says **0.476** on RV and 0.609 on OVX. The market
is implying ~27% vol on a contract whose 14-day realized is 48.8% and whose listed IV is
68. One of us is badly wrong and it resolves in five sessions — which makes it the single
most informative row we can record this week. Buys stay disabled.

### Proposed rows — `results/proposed-rows-2026-07-26.csv`, 13 rows

Emission rule, stated in advance and applied uniformly: **two-sided CLOB book, spread ≤
5c, mid ∈ [3c, 97c]**. That deliberately drops the sub-3c wings this variant used to
emit — they are the category the executable-price audit showed to be unsellable, and
adding more of them re-inflates a headline the firm has already agreed to stop quoting.

| board | rows |
|---|---|
| WTI July monthly | ↓75, ↓80, ↓85, ↓90 (from Jul 25), ↑95, ↑100, ↑105 |
| Gold July monthly | ↓3900, ↑4300 |
| Silver July monthly | ↓52, ↓54, ↑64, ↑66 |

All 13 resolve **2026-07-31 21:00Z**. Zero rows on: August WTI/gold/silver (no book — see
the roll write-up), all five week-of-Jul-27 boards (no book, or no tape).

Note for the CEO: the `model` column reads `claude-opus-5`, the exact id. Earlier rows in
the ledger say `opus-5` / `opus` / `fable`; normalise on append if you want a single key.
