# Coding guidelines

For every agent writing code in this repo.

## Style

- **Straightforward procedural/functional code.** Data in, data out. Plain functions over
  traits, flat modules over deep hierarchies, duplication over premature abstraction.
  Complex abstraction is a relic of an era when change was expensive — for us, rewriting is
  cheap and indirection is what's expensive.
- **Think big, don't de-risk like a human.** Humans hedge because their time is scarce;
  you can produce a lot of code quickly. Prefer the radical rewrite that reaches the best
  possible result over the timid incremental patch that preserves a mediocre one.
- **Copy freely.** No shared internal library. Reuse happens via wiki recipes, templates,
  and copying working code between variants. Only *conventions* (schemas, folder layout)
  are single-source.
- **Try hard.** A strategy variant deserves real modeling — calibrated simulations,
  out-of-sample backtests, proper distributions — not a heuristic and a shrug. Depth of
  research is the product.

## Rust (the nudged default)

- One-off scripts: inline `cargo -Zscript` single files with embedded deps (nightly).
- Anything larger or long-lived (a variant's model, tooling): a real cargo crate.
- Other languages are allowed when clearly better for the job — say why in the worklog.

## Practical

- Snapshot every dataset you rely on (via `tools/r2data/`); never depend on live data for
  reproducing a result.
- No secrets in code or git — environment variables only.
- Commit + push after every logical step.
