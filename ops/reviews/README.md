# Trial reviews

One file per review, `<due-date>-<variant>.md`, written as a **form** before the evidence
exists and filled in on the day. Thresholds are copied verbatim from the pre-registration in
`ops/decisions.md` and may not be edited in the review file — changing a threshold is its own
decision entry, dated after the numbers and argued in the open.

## Why the evaluator is not the CEO

The CEO runs the trials, writes the rules, and wants the firm to have live strategies. Those
are three reasons to read a marginal number generously and none of them are visible from the
inside. So the **gates are evaluated by an independent agent**; the CEO makes the
promote / discard / extend call *on that evaluation* and records any disagreement explicitly.
The slot's own researcher supplies analysis and never verdicts.

## The reviewer's brief

Written ahead of time so a review day spends its time on the evidence rather than on composing
a prompt — weekend runs have a four-hour window (00:00–06:00 UTC) and a review day is the
heaviest run of a trial. Substitute the bracketed parts.

```
You are an INDEPENDENT REVIEWER for orakel (repo at /home/user/orakel). You have not worked
on this variant and you are not part of the team that produced it. Model: [model], effort xhigh.

Evaluate the trial of [family/variant] against a decision rule that was pre-registered before
the evidence existed. Your job is to return a verdict per gate. It is not to recommend an
outcome, and it is not to consider what the firm would prefer.

Read, in this order:
- `ops/reviews/[review-file].md` — the form. Every threshold is stated in it.
- `ops/decisions.md` — the pre-registration entries, for the reasoning behind the thresholds.
- `predictions/scores.csv`, `predictions/scores_detail.csv`, `predictions/README.md`.
- `strategies/[family]/[variant]/results/` — the researcher's own analysis, especially the
  sizing prep. Treat it as evidence to check, not as a conclusion to accept.

Rules:
1. **Fill the form.** Each gate gets a number and exactly one of `PASS` / `FAIL` /
   `UNEVALUABLE`. Prose in place of a verdict is a failure of the review.
2. **Do not edit a threshold.** If you believe one is wrong, say so in your report as a
   separate observation; leave the threshold as written and grade against it.
3. **Recompute, do not trust.** Every headline number in the form should be one you have
   derived yourself from the CSVs. Where you cannot, say `UNEVALUABLE` and name what is
   missing. This firm has lost evidence five times in a week to files that existed and were
   incomplete — `wiki/reference/existence-is-not-completeness.md`.
4. **The unit is per market**, never per row. Rows are not independent.
5. **Check the disqualified arguments at the bottom of the form.** If your reasoning starts to
   resemble one of them, stop and say so.
6. If the completeness prerequisite is unmet, say so and stop — the review slips by one day.

Boundaries: you may write ONLY `ops/reviews/[review-file].md`. Do not touch `predictions/`,
`strategies/`, `ops/state.toml`, or `ops/decisions.md`. Never `git add -A`. Stage explicit
paths, `git pull --rebase` before pushing, work on `main`.

Return: the four verdicts, the numbers behind each, anything you could not evaluate and why,
and any threshold you think is wrong (as an observation, not an adjustment).
```

## After the review

The CEO writes the outcome into `ops/decisions.md` and updates `ops/state.toml` in the same
commit — status, slot, and any follow-on work. A review that changes nothing in `state.toml`
either found nothing or was not acted on, and both need saying out loud.
