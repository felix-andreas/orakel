---
from: ceo
to: felix
date: 2026-07-25
status: open
subject: Dashboard is live-reading, but the GITHUB_TOKEN you provisioned is a broad classic PAT
---

`GITHUB_TOKEN` is now set as the `orakel-dashboard` Worker secret and live repo reads work
(verified: the dashboard's freshness stamp tracked a commit pushed *after* the deployed
build, so it is reading `main` at request time, not the embedded pack).

One thing to decide, because the token is stronger than the dashboard needs. Queried from
the session, the token authenticates as `felix-andreas` with scopes:

```
admin:public_key, gist, read:org, repo
```

That is a **classic PAT with `repo` write access to all your repositories**. `dashboard/README.md`
and `wrangler.toml` both specify a *fine-grained PAT with read-only Contents on
felix-andreas/orakel* — the dashboard only ever issues GETs (Contents, Trees, one commit
read), so read-only Contents is sufficient.

The gap matters because a Worker secret is readable by anything that can deploy to the
Cloudflare account, and this one would grant repo-write on everything you own rather than
read on one repo.

Suggested (your call, no rush — the dashboard works either way):

1. Create a fine-grained PAT: repository access = `felix-andreas/orakel` only, permission
   = *Contents: Read-only*, with an expiry you're happy to rotate.
2. `cd dashboard && npx wrangler secret put GITHUB_TOKEN` and paste it.
3. Revoke/narrow the classic PAT if it exists only for this.

No verification of the swap is needed beyond loading any page and checking the top bar
still says `live`.

## Reply (appended by recipient, with date)
