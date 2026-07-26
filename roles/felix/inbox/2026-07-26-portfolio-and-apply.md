---
from: ceo
to: felix
date: 2026-07-26
status: open
subject: Two choices on the paper book before "apply" can mean anything — bankroll size, and whether the dashboard may write
---

Building the Execution surface you asked for (holdings overview + terraform-style plan/apply).
The plan half needs nothing from you. The other half has two decisions I should not take
alone, and one of them is a security decision.

## 1. Apply requires a write path the dashboard deliberately does not have

The dashboard is a read-only Worker. It reads the repo from GitHub at request time and cannot
commit — that is by design, and yesterday we removed its only local copy of the repo precisely
so it could never serve state it had invented.

**Plan is unaffected.** It is a pure function of (your selection, the current book, current
prices), the selection lives in the URL, and it writes nothing — so a plan is safe to
generate, share as a link, and re-run. That is also the valuable half in terraform, and it
works today.

**Apply is the question.** Three options:

- **(a) Apply renders the rows; the CEO commits them.** Ships now, no new access. You click
  apply, the page shows the exact `ledger.csv` rows and a one-line command, and the next CEO
  run commits them. Auditable by construction, because applying is a deliberate act with a
  commit attached. Costs you a round trip.
- **(b) Give the Worker repo write access.** Apply commits directly. This means a web-facing
  Worker behind Cloudflare Access holds a token that can write to the firm's own state. Note
  the token you provisioned is a **classic PAT with `repo` write on all your repositories** —
  I flagged that on 07-25 and it is still open. I would not do this without a fine-grained
  token scoped to `orakel` contents only, and even then it is your call.
- **(c) Apply writes an intent file via a narrow endpoint** — a middle path: the Worker can
  append to one file and nothing else. More moving parts than (a) buys little over it.

**I have built for (a)** because it ships today and takes no new permissions. Say the word
and I will do (b) with a properly scoped token, or leave it at (a).

## 2. Bankroll is a number I invented

`portfolio/portfolio.toml` has `starting_cash = 10000.0` USDC. It is a notional placeholder
so percentages mean something — return on locked capital, capital efficiency, "what fraction
of the book is deployed" are all meaningless without a denominator.

Nothing depends on the number except the scaling, and changing it in that one file rescales
the whole book. But if you have a figure in mind — the size you would actually consider
deploying if this ever graduates past paper — tell me and I will set it, because it changes
which trades the policies consider worth taking at all. A policy that earns 40% annualised on
3% of a $10k book is a different proposition from the same policy on a $1M one.

## For context on why this surface matters

The backtest already told us the most important thing we know: on our own live signals,
**seven of the eight policies take zero trades**. Being calibrated and being tradeable are
different things. The book is where that stops being a retrospective statistic and starts
being a running claim — if a policy would hold nothing this week, the holdings tab will say
so, in the open, every day.
