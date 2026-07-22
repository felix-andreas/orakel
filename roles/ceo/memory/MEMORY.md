# CEO Memory

_Keep under ~150 lines. Prune every run. Details go to worklogs / run manifests / wiki._

## Short-term (current run / immediate)

- Firm founded 2026-07-22. Nothing has run yet. First real run: verify toolchain, check
  secrets (`ops/state.toml [secrets]`), spawn market researcher, fill first slot.

## Medium-term (bootstrap phase)

- Ramp plan: 1–2 slots until a full day runs clean, then scale toward 5.
- Dashboard needs first deploy once `CLOUDFLARE_API_TOKEN` exists (then Access setup by
  Felix; then `GITHUB_TOKEN` as Worker secret via wrangler).

## Long-term (durable principles)

- Constitution: observability first, spend logged, working window, model routing, no
  trading, single-writer CSV, R2-before-commit, commit+push every step.
- Inherited from poly (scored evidence): consensus/combination beats individual signals;
  model choice matters (record exact model ids); escalate-on-flag works; agents can die
  silently mid-run — always audit folders before assuming loss.
