# Favorite-longshot bias inside bucket families

The classic betting bias — longshots overpriced, favorites underpriced — shows up *inside*
mutually-exclusive Polymarket bucket families even when the family arithmetic is clean.
Because legs must sum to ~1, every cent of longshot premium is funded by shaving the
favorites: a family can pass a de-vig/sum check perfectly and still be internally
misallocated. **Sum coherence tests vig; it does not test allocation.**

## Detection

1. De-vig the family (normalize live-leg YES mids to sum 1).
2. Build an *independent* model of the legs, ideally anchored to sharper sibling markets
   or bookmaker lines.
3. Compare per-leg ratios (market/model). Signature: tail legs at 1.5–5×+ model, the 1–2
   favorite legs at 0.90–0.97×, mid legs mixed. Suspect it especially where a tail leg
   carries a **narrative** (star player, recent headline).

Observed (poly, 2026 WC top-scorer family): two independently-anchored sims agreed the
Spain leg traded ~2–5× model, Norway ~1.5–2× (a "Haaland premium" on a leg needing an
upset *and* a 4-goal comeback), funded by Argentina and France; family summed 0.9915
throughout. Bonus: an eliminated team's leg printed 0.005 for hours after becoming a
mathematical zero — check losers' legs for free-zero windows after each elimination.

## How to use it (and how not to)

- **Not usually a ticket**: the premium is cents on thin tail books; after spread the
  "sell the longshot" side rarely executes.
- **Not favorite edge either**: the mirrored favorite discount (1–3c) sits inside the
  favorite's own spread.
- **Real uses**: (a) when de-vigging a family into an implied distribution, expect tails
  fat by ~2× — correct before using it as a model input; (b) a favorite leg below your
  model in a family with pumped tails is weak evidence *for* your model; (c) tail legs are
  the last place to look for value and the first place to look for exit liquidity.
