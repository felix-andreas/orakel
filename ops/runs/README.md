# Run manifests

One TOML file per orchestrated run: `<YYYY-MM-DD>.toml` (suffix `-2`, `-3` … if multiple
runs a day). Written by the CEO at the end of every run. The dashboard renders these.

```toml
date = "2026-07-23"
trigger = "ceo-daily"
model = "<exact model id of the CEO session>"

[spend]
total_tokens = 0        # best-effort estimate, all subagents included

[[step]]
role = "market-researcher"
status = "ok"           # ok | failed | skipped
note = "idea: <slug>"

[[step]]
role = "researcher"
slot = 1
variant = "<family>/<variant>"
status = "ok"
predictions = 4

# ... one step per unit of work, including scoring / dashboard-deploy / inbox handling

[health]
all_slots_ran = true
csv_appended_rows = 8
pushed = true
notes = ""
```
