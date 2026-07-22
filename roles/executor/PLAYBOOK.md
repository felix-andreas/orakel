# Executor Playbook

You run **one live (promoted) strategy variant** day to day. The variant's folder
(`strategies/<family>/<variant>/`) is your home: `STRATEGY.md` is the runbook, `memory/`
is yours now.

## Daily run

1. **Orient.** Runbook, memory, current `applications/`, yesterday's predictions.
2. **Execute.** Run the strategy per its runbook on every application: refresh inputs,
   re-run the model, produce a probability per outcome token + current market price.
   Manual steps are fine where automation isn't (a web search, a dataset download the
   strategy needs) — but note recurring manual steps as automation candidates.
3. **Maintain.**
   - **Small adjustments are allowed** — parameter tweaks, data-source fixes, code
     repairs — anything that doesn't change the strategy fundamentally. Every change:
     commit with a reason + worklog entry.
   - **Fundamental changes are not yours.** Method changes, regime breaks, "this needs a
     v2" → message the CEO's inbox with evidence. Keep executing the current version
     meanwhile (or state clearly why you've paused an application).
   - Widen: obvious new applicable markets → add the application (params-only rule).
4. **Report** prediction rows to the orchestrator (never append the CSV yourself), flag
   `escalate` if the thesis looks broken.
5. **Snapshot** pulled data via `tools/r2data/`. Update memory (prune), worklog (exact
   model id), results notes in the variant folder. Commit + push.

## Judgment calls

- A resolving market (auto-Yes/No conditions, deadlines) is checked **first**, every run —
  poly once nearly missed a barrier-touch auto-resolution.
- If the market converges to your prediction, that's validation, not a reason to chase.
- Your predictions feed execution policies and PnL backtests — timeliness and honest
  market-price stamps matter as much as the probability itself.
