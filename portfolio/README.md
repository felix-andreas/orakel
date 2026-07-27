# portfolio — the paper book

> **In plain English:** this is a pretend trading account. It records what the firm *would*
> be holding if it acted on its own signals, and how much pretend cash is left. No real
> money moves, no order is ever sent, and nothing here touches an exchange.

`CONSTITUTION.md` makes real trading a hard line. This directory exists so that the gap
between "we were right" and "we would have made money" is a thing you can look at rather
than argue about. Everything is marked paper, everywhere it is shown.

## Why a book at all, when we already have a backtest

They answer different questions and neither substitutes for the other.

- **Backtest** (`execution/`) replays *stored* signals against *stored* prices to compare
  policies over history. It is retrospective and it is where policy choice is decided.
- **The book** (here) is the current state: what we hold *now*, at *today's* prices, with a
  specific policy actually chosen. It is what makes a policy's claim falsifiable going
  forward rather than only in sample.

The backtest told us the most important thing we know — on our own live signals, seven of
eight policies take **zero** trades. The book is where that stops being a statistic.

## Files

### `portfolio.toml` — the account

```toml
[account]
opened = "2026-07-26"
currency = "USDC"
policy = ""               # the execution policy this book is bound to, e.g. "harvest-v2"
                          # empty = no policy chosen yet; the plan page picks one per plan

[sizing]
notional_bankroll = 10000.0   # what policies size against. NOT money anyone owns and
                              # NOT part of NAV — see "The opening balance has no owner"

[marks]
source = "clob-midpoint"  # how open positions are marked. A midpoint is NOT a fill price
                          # (wiki/reference/midpoint-is-not-a-fill.md) — see "Marks" below.
```

### `positions.csv` — what is open right now

```
opened_at,market_slug,condition_id,outcome,token_id,side,shares,entry_price,collateral,family,variant,policy,run_id
```

- `side` — `buy` or `sell` of the named `outcome` token.
- `collateral` — cash locked by this position. Buying costs `shares × entry_price`; **selling
  YES at p locks `shares × (1 − p)`**, which is the rule the whole firm's accounting turns on
  (`execution/DESIGN.md` §3). Cents-per-trade flatters exactly the trades we should refuse.
- One row per open position. A position that closes moves to `ledger.csv` and leaves.

### `ledger.csv` — append-only history of everything applied

```
applied_at,action,market_slug,condition_id,outcome,token_id,side,shares,price,fee,cash_delta,collateral_delta,policy,plan_id,note
```

- `action` — `open` | `increase` | `reduce` | `close` | `resolve` | `deposit`.
- `fee` — the venue taker fee actually modelled: `shares × rate × p × (1 − p)`, taker-only,
  charged on entry AND on a market exit, never at resolution (`execution/DESIGN.md` §4).
  Geopolitics is 0; finance/politics/tech 0.04; crypto 0.07; most else 0.05.
- Append-only. `positions.csv` and the cash figure are **derived** from this file and must be
  reproducible from it alone — if they ever disagree, the ledger wins.

## Cash

`cash = Σ contributions − Σ redemptions + Σ cash_delta`. Free cash is `cash − Σ collateral` over open
positions. Both are shown, because a book with no free cash is fully deployed even if its
cash line looks healthy — and `execution/DESIGN.md` §3 already makes the point that a policy
earning 40% on 3% of the bankroll is a rounding error on the fund.

## Marks

Open positions are marked at the CLOB midpoint, and **the midpoint is not a price you can
get**. Our own first scored batch beat the market 21/21 and had a counterparty at the scored
price on 2 of 21 (`wiki/reference/midpoint-is-not-a-fill.md`). So any unrealised P&L shown
here is an upper bound and must be labelled as one. Where `predictions/fills.csv` has an
observed bid for a position, show the mark-to-bid figure beside the mark-to-mid one — that
pair is the honest picture and the difference between them is the point.

## The opening balance has no owner — so it is not in NAV

The first version of this schema had `starting_cash = 10000` **and** "the first contribution
strikes at 1.0000". A dashboard agent caught that those cannot both hold: NAV would be 10,000
against 0 shares, so a first contribution of 1,000 buys 1,000 shares at 1.0000 and the
contributor instantly owns an 11,000 book. The first person in is handed the entire opening
balance and everyone after them buys at 11× what the money supports. That was a CEO design
error, not a rounding artifact, and it does not shrink until real contributions dilute it.

The mistake was conflating two different numbers:

| | what it is | in NAV? |
|---|---|---|
| **`sizing.notional_bankroll`** | the size policies compute positions against, so a plan can say anything at all | **no** |
| **contributions** | money someone actually paid in, recorded in `investors.csv` | **yes** |

So: **`notional_bankroll` is a sizing parameter and never enters NAV, cash or shares.** It has
no owner because nobody paid it. NAV is contributions plus what they earned, which is zero
until somebody contributes — and a book with zero shares has **no** NAV per share, rather than
a NAV per share of 1.0000. The page says "no shares issued" rather than inventing a price.

This keeps both properties we wanted: policies can still size (the plan page works on an empty
book), and the first contributor gets exactly what they paid for.

Felix's real bankroll figure is still open in
`roles/felix/inbox/2026-07-26-portfolio-and-apply.md`; it changes what policies consider worth
trading, and nothing else.

## Plan and apply

The plan page is a **pure function**: given a selection (strategies × markets × policy), the
current book, and current prices, it computes target positions and the diff to reach them.
It writes nothing, so a plan is safe to generate, link and re-run — the selection lives in
the URL and a plan is therefore reproducible by anyone with the link.

**Apply is deliberately not automatic.** The dashboard is a read-only Worker: it reads the
repo from GitHub at request time and has no write path, by design. Granting a web-facing
Worker commit access to the firm's own state is a security decision for Felix, not a
convenience for me to take — so today "apply" renders the exact rows to append and the CEO
commits them on the next run. That is also the more honest shape: an apply that requires a
deliberate act leaves an auditable trail, which is what the ledger is for.

## Shares, NAV and the "Last" price — the book is a fund, not one person's account

Several people can pay into the same book. That makes it a pooled vehicle, so ownership is
tracked in **shares**, not in each person's dollars, and the whole thing turns on getting one
number right.

### `investors.csv`

```
at,investor,action,amount_usd,nav_per_share,shares_delta,note
```

- `action` — `contribute` | `redeem`.
- `shares_delta = amount_usd / nav_per_share`, positive on a contribution, negative on a
  redemption. **The rate used is the NAV per share at that moment**, which is why the rate is
  stored on the row: it is the evidence for how many shares someone got.
- The first contribution into an empty book strikes at `nav_per_share = 1.0000` by
  definition — and it is honest precisely because `notional_bankroll` is not in NAV, so
  1.0000 corresponds to a book holding exactly what that first contributor paid in.

### The three numbers

| | |
|---|---|
| **NAV** | `cash + Σ unrealised P&L on open positions`, where cash is contributed money only — `notional_bankroll` is excluded. Cash already includes money committed as collateral (posting collateral commits money, it does not spend it), so this does not double count. |
| **Shares** | `Σ shares_delta` over `investors.csv`. |
| **NAV per share ("Last")** | `NAV / shares` — **undefined while shares are zero**, and shown as "no shares issued" rather than as a number. |

### The thing that must not be got wrong

NAV depends on how open positions are marked, and **a midpoint is not a price you can get**
(`wiki/reference/midpoint-is-not-a-fill.md` — our own first scored batch beat the market 21/21
and had a counterparty at the scored price on 2 of 21).

With one account that is a reporting nicety. With several people it is a fairness problem, and
it runs in both directions:

- Issue shares at a midpoint-based NAV that the book could not actually realise, and the new
  investor buys in at an inflated price — they are diluted the moment anyone checks.
- Redeem at that same inflated NAV and the *remaining* holders pay the difference.

**So share issuance and redemption use the conservative mark: what the book could be
liquidated at today, hitting the bid on longs and lifting the ask on shorts.** The
midpoint-based NAV is shown beside it, labelled as the optimistic bound. Where the two differ,
the gap is displayed — that gap is the honest measure of how much of the book's stated value
is real, and it belongs on the page rather than in a footnote.

Every share-issuing action records the NAV it used, so any dilution question can be settled
from the file rather than from memory.
