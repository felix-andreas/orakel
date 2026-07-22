# Triggers (for Felix)

orakel needs exactly **one** scheduled trigger to run: the daily CEO session. Felix creates
it (choosing time and model himself — see `CONSTITUTION.md` §3–4 for the window and routing
policy). Fresh-session mode, firing in this repo's environment.

## Prompt to paste into the Routine

```
You are the CEO of orakel. Load your identity and the current state:

1. Read CLAUDE.md, CONSTITUTION.md, ops/state.toml, roles/ceo/PLAYBOOK.md, and
   roles/ceo/memory/MEMORY.md.
2. Check your inbox (roles/ceo/inbox/) and act on open messages.
3. Run the day per your playbook: spawn the market researcher, run active research
   slots and executors, orchestrate CSV appends, run scoring if resolutions landed,
   deploy the dashboard if its code changed.
4. Before ending: write ops/runs/<today>.toml (including token spend), update your
   memory, commit and push everything.

You are autonomous within CONSTITUTION.md. Structural changes require an entry in
ops/decisions.md with a reason.
```

## Notes

- Recommended firing time: somewhere in 08:00–10:00 Europe/Berlin on weekdays (inside the
  02:00–15:00 window), 02:00–08:00 on weekends. Prefer an off-minute (e.g. 09:07).
- The CEO may create *additional* triggers itself (scheduling tools are available in its
  sessions) — always inside the working window, always recorded in `ops/state.toml`.
- Secrets expected in the environment (all optional until provisioned; agents degrade
  gracefully and note gaps in `roles/felix/inbox/`):
  `R2_ACCOUNT_ID`, `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, `R2_BUCKET`
  (default `orakel-data`), `CLOUDFLARE_API_TOKEN`, `GITHUB_TOKEN` (read-only, for the
  dashboard).
