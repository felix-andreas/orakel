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
  `R2_ACCOUNT_ID`, `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`,
  `CLOUDFLARE_API_TOKEN`, `GITHUB_TOKEN` (read-only, for the dashboard).
  The bucket name is NOT an env var — `orakel` is hard-coded as the default everywhere
  (`R2_BUCKET` exists only as an optional override).

## Recommended permission allowlist (Felix-only change)

Agents cannot edit `.claude/settings.json` themselves (the permission classifier
hard-blocks it — by design). For friction-free autonomous runs, Felix should extend the
`permissions.allow` array to:

```json
[
  "mcp__Claude_Code_Remote__create_trigger",
  "mcp__Claude_Code_Remote__update_trigger",
  "mcp__Claude_Code_Remote__delete_trigger",
  "mcp__Claude_Code_Remote__list_triggers",
  "mcp__Claude_Code_Remote__fire_trigger",
  "mcp__Claude_Code_Remote__send_later",
  "mcp__Cloudflare_Developer_Platform__r2_buckets_list",
  "mcp__Cloudflare_Developer_Platform__r2_bucket_get",
  "mcp__Cloudflare_Developer_Platform__workers_list",
  "mcp__Cloudflare_Developer_Platform__workers_get_worker",
  "mcp__Cloudflare_Developer_Platform__search_cloudflare_documentation",
  "Bash(git *)",
  "Bash(cargo *)",
  "Bash(rustup *)",
  "Bash(npx wrangler *)"
]
```

(Deliberately excluded: bucket/database create+delete and KV/D1 writes — those should
stay prompt-worthy or CEO-requested via inbox.)
