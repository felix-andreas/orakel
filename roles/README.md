# Roles

A role is a **playbook + memory + inbox**, not a conversation. Any session that loads
`roles/<role>/PLAYBOOK.md` and the role's memory *is* that role; continuity lives entirely
in git. Researchers and executors are instantiated per strategy variant — their memory
lives in the variant's folder, their playbooks here.

```
roles/<role>/
├── PLAYBOOK.md   # how the role operates
├── memory/
│   ├── MEMORY.md   # short / medium / long-term bullets; PRUNE aggressively
│   └── WORKLOG.md  # one dated entry per run, naming the exact model id
└── inbox/          # incoming messages (see format below)
```

## Memory rules

- **Short-term**: this run's state — overwritten freely, pruned every run.
- **Medium-term**: valid for the current phase/strategy life.
- **Long-term**: durable principles. Candidates for the wiki.
- **Format: bullet points.** Anything longer than a few lines becomes its own file in
  `memory/`, linked from the bullet — `MEMORY.md` stays a scannable index, not an essay.
- **Budget: keep `MEMORY.md` under ~150 lines.** poly's CEO memory became an unreadable
  scroll of daily logs; details belong in worklogs, run manifests, linked notes, and the
  wiki, not in memory. Prune every run — superseded bullets die.
- **Memory lives in the repo, always.** Session storage is ephemeral; a memory that isn't
  committed and pushed doesn't exist.

## Inbox message format

One markdown file per message: `roles/<role>/inbox/<YYYY-MM-DD>-<slug>.md`

```markdown
---
from: executor:barrier-touch/gbm
to: ceo
date: 2026-08-01
status: open        # open | answered | rejected | done
subject: Request v2 — vol regime shifts break the calibration
---

Body: what, why, what you propose. Keep it short; link to evidence.

## Reply (appended by recipient, with date)
```

Rules: the recipient owns the status field; replies are appended to the same file, so each
file is a complete thread. Handling the inbox is the first step of every role's run. The
dashboard renders all inboxes; `status: open` items older than 7 days are flagged.

## Current roles

| Role | Playbook |
|------|----------|
| CEO | [`ceo/PLAYBOOK.md`](ceo/PLAYBOOK.md) |
| Market researcher | [`market-researcher/PLAYBOOK.md`](market-researcher/PLAYBOOK.md) |
| Researcher (per slot) | [`researcher/PLAYBOOK.md`](researcher/PLAYBOOK.md) |
| Executor (per live variant) | [`executor/PLAYBOOK.md`](executor/PLAYBOOK.md) |
| Felix (human) | inbox only: [`felix/inbox/`](felix/inbox/) |

New roles may be created by the CEO (structural change → `ops/decisions.md`).
