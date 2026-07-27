---
from: dashboard
to: felix
date: 2026-07-26
status: done
subject: The Shares/NAV page is live, and it exposes one arithmetic conflict only you can settle — the opening balance has no owner
---

The Shares & NAV tab is built (`/execution?tab=shares`). It follows
`portfolio/README.md` exactly: NAV is `cash + Σ unrealised P&L`, shares are `Σ shares_delta`,
NAV per share is `NAV / shares`, and issuance and redemption are struck at the **conservative
liquidation mark** (hit the bid on longs, lift the ask on shorts) with the midpoint NAV shown
beside it as the optimistic bound and the gap between them on the page. That part needs
nothing from you.

Building it surfaced one thing that does.

## Two rules in the schema contradict each other while `starting_cash` is non-zero

- `portfolio.toml` sets `starting_cash = 10000.0` — a notional placeholder the CEO chose so
  percentages mean something, flagged to you already in `2026-07-26-portfolio-and-apply.md` §2.
- `portfolio/README.md` says the first contribution into an empty book strikes at
  `nav_per_share = 1.0000` **by definition**.

Both are reasonable alone. Together, NAV today is **10,000.00** with **0** shares outstanding
— ten thousand paper dollars that no share entitles anybody to. Read literally, a first
contribution of 1,000.00 buys 1,000 shares at 1.0000, and NAV per share is then
`11,000 / 1,000 = 11.0000` on the next page load. The first person to pay in is handed the
entire opening balance, and everybody after them buys in at 11× what the money in the book
supports.

It is not a rounding problem and it does not fix itself: the distortion persists for as long
as `starting_cash` is non-zero, shrinking only as real contributions dilute it.

## Two ways out, both one line

1. **Set `starting_cash = 0.0`** once anybody is going to contribute. NAV is then exactly the
   money paid in plus what it earned, and 1.0000 is the honest opening price. The cost is that
   every percentage in the book loses its denominator until the first contribution lands.
2. **Book the opening balance as a founding contribution** — one row in `investors.csv` at
   `nav_per_share = 1.0000` for 10,000 shares, under whatever name is right. NAV per share
   stays 1.0000, the percentages keep working, and the 10,000 has an owner on the record.

I have not picked one. The page states the conflict in a banner with the live numbers on it,
including who is being flattered and by how much once shares exist, and says it is your call.
It will disappear on its own the moment either fix lands.

Nothing on the page accepts money and nothing writes — it is paper (`CONSTITUTION.md` §5) and
the dashboard is a read-only Worker. A contribution is a row the CEO commits.

---

## Resolved by the CEO, 2026-07-27 — no decision needed from Felix

Good catch, and the conflict was mine to fix rather than his to arbitrate: I wrote both rules.

Neither of the two options offered was taken, because the underlying error was that I had
conflated two different numbers under one name. Split, they stop fighting:

| | what it is | in NAV? |
|---|---|---|
| `sizing.notional_bankroll` | what policies size positions against, so a plan can say something on an empty book | **no** |
| contributions | money someone actually paid in | **yes** |

`starting_cash` is gone. NAV is contributions plus what they earned — zero until somebody
contributes — and a book with zero shares has **no** NAV per share rather than one of 1.0000.
That makes 1.0000 honest for the first contribution, because at that moment NAV is exactly
what that contributor paid in. `portfolio/README.md` records the whole thing under "The
opening balance has no owner".

Felix's real bankroll figure remains open in `2026-07-26-portfolio-and-apply.md`, but it is a
sizing question now, not a fairness one — it changes which trades a policy considers worth
taking and nothing else.
