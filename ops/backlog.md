# Firm backlog

Work items for the firm itself — tooling, process, capability. **Not** market ideas; those
live in `ideas/` and are tracked in `ops/idea-funnel.md`.

Maintained by the CEO. One item per heading, newest first. An item leaves this file by being
done (with a line saying where the work landed) or by being explicitly dropped with a reason.
The daily run picks from here when there is capacity — see `roles/ceo/PLAYBOOK.md`.

---

## Autonomous paper trading, separate from the human-guided book

**From Felix, 2026-08-02.** The firm should run its own paper trading, independent of the
human-guided flow on `/execution` (Plan → Apply, which Felix drives).

What this means and what it needs, as I currently understand it:

- **Allowed.** `CONSTITUTION.md` §5 forbids *real* trading — no wallets, no order signing, no
  exchange keys. Paper is untouched by that, and the execution layer was built paper-only from
  day one for exactly this.
- **Distinct from the existing book.** The `/execution` page is a *proposal* surface: it plans a
  diff and waits for a human to apply it. This item is the firm deciding and recording its own
  positions on its own cadence, with the human-guided book left as-is.
- **Depends on two open Felix items**: the apply-write decision and the bankroll figure
  (`roles/felix/inbox/2026-07-26-portfolio-and-apply.md`). An autonomous book needs a size to
  trade against and permission to write its own ledger.
- **Blocked in practice on having something to trade.** `barrier-touch/ladder-rv` is under
  review and its funding bound fails at the executable price; if it is discarded there is no
  live signal to trade at all. So build the mechanism, but do not pretend a book is meaningful
  before a variant clears its gates.

Open question for me: whether the autonomous book trades *only* promoted (`live`) variants, or
also trials in shadow. Shadow-trading a trial is the more informative version and costs nothing
in paper, but it needs its own ledger so trial rows never contaminate the live record.

## The daily run should work the backlog when it has capacity

**From Felix, 2026-08-02.** The daily trigger currently does: orient → inboxes → market
researcher → slots → scoring → health → close. When there is capacity and this file is
non-empty, it should also pull an item from here.

Why it matters now: with one trial under review and four empty slots, the binding constraint
has not been researcher capacity for a week. Several real improvements have been carried in
`[next_run]` blocks of run manifests instead — which scatter, and which nobody reads as a list.

Needs: a step in `roles/ceo/PLAYBOOK.md` naming this file, and a rule for how much of a run may
go to backlog work without starving the research cadence.

## Dashboard: the cold-cache read loss

Carried from `roles/ceo/inbox/2026-07-29-dashboard-cold-cache-reads.md`. Three hypotheses tried,
all disproved; instrument a cold request before changing anything else. Fires on the first
request after a push or deploy, and gets worse as `ops/runs/` grows.
