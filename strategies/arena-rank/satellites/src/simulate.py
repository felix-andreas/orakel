"""Order-statistic simulation of the arena ranking, for pricing a whole board cohort.

A board asks: which company owns the k-th ranked model in slice S at instant T. That is an
order statistic over *company portfolios* - a company holding several models near the top
wins through a max over correlated scores - so the cohort must be simulated jointly from
one latent ranking, not priced leg by leg.

Two sources of randomness, both measured from the vintage panel rather than assumed:

  drift     each incumbent's PUBLISHED score moves between now and the check. Measured sd
            for top-25 models is ~1.2 pts at 7d, ~1.8 at 14d, ~2.6 at 30d - far tighter
            than the published +/-CI (mean +/-5.9, i.e. sd ~3.0). The CI describes
            uncertainty about latent skill; the market resolves on the printed number.
            Using the CI as sigma overstates rank uncertainty by ~2.5x.
  entrants  models that do not exist yet: ~7.7 new top-20 models per 30 days, ~2.5 of them
            top-5. This dominates, and it is NOT modellable from public data (release
            timing is an insider process), so it is drawn from the empirical arrival and
            relative-score distribution.

Vectorised with numpy: the calibration grid needs ~10^7 simulated rankings.
"""

import json
import os
import sys
from collections import Counter

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from resolve import CHINESE, norm  # noqa: E402

# sd of published-score change for top-25 models, by horizon in days (see drift.py)
DRIFT_KNOTS = np.array([[0, 0.0], [7, 1.25], [14, 1.80], [30, 2.60], [45, 6.00]])


def drift_sd(h_days, scale=1.0):
    return float(scale * np.interp(max(0.0, h_days), DRIFT_KNOTS[:, 0], DRIFT_KNOTS[:, 1]))


def load_spec(path):
    d = json.load(open(path))
    recs = d["entrants"]
    return dict(
        rate_top20_per_30d=len(recs) / d["span_days"] * 30.0,
        rel_scores=np.array([r["rel"] for r in recs], dtype=float),
        orgs=dict(Counter(r["org"] for r in recs)),
    )


def simulate(rows, horizon_days, spec, n_sims=20000, drift_scale=1.0, rate_scale=1.0,
             top_m=60, seed=0, incumbency=0.7, max_new=8):
    """Return {1,2,3,'chinese1'} -> {company: probability}.

    `incumbency` mixes the entrant-company distribution towards companies already in the
    current top-20. Drawing an entrant's company unconditionally is wrong and expensive:
    most "new" models are new variants from whoever is already on top
    (claude-opus-4-6 -> claude-opus-4-6-thinking), so an entrant usually does NOT change
    which company owns rank k. That correlation is the portfolio effect.
    """
    rng = np.random.default_rng(seed)
    inc = sorted(rows, key=lambda r: (r["rank"], -r["score"]))[:top_m]
    if len(inc) < 5:
        return None
    scores = np.array([r["score"] for r in inc], dtype=float)
    orgs = [r["org"] for r in inc]
    best = scores.max()
    sd = drift_sd(horizon_days, drift_scale)

    # company index space: incumbents + any historical entrant company
    names = list(dict.fromkeys(orgs + sorted(spec["orgs"])))
    idx = {c: i for i, c in enumerate(names)}
    inc_ix = np.array([idx[o] for o in orgs])

    top20 = Counter(orgs[:20])
    t20 = sum(top20.values()) or 1
    glob = spec["orgs"]
    tg = sum(glob.values()) or 1
    w = np.array([
        incumbency * top20.get(c, 0) / t20 + (1 - incumbency) * glob.get(c, 0) / tg
        for c in names
    ])
    w = w / w.sum()

    lam = spec["rate_top20_per_30d"] / 30.0 * rate_scale * horizon_days
    n_new = int(min(max_new, max(1, np.ceil(lam + 3 * np.sqrt(max(lam, 1e-9))))))

    M = len(inc)
    S = np.empty((n_sims, M + n_new))
    C = np.empty((n_sims, M + n_new), dtype=np.int32)

    S[:, :M] = scores[None, :] + rng.normal(0, sd, size=(n_sims, M))
    C[:, :M] = inc_ix[None, :]

    if n_new:
        rel = spec["rel_scores"]
        draws = rel[rng.integers(0, len(rel), size=(n_sims, n_new))]
        S[:, M:] = best + draws + rng.normal(0, sd, size=(n_sims, n_new))
        C[:, M:] = rng.choice(len(names), size=(n_sims, n_new), p=w)
        # thin the entrant slots down to a Poisson(lam) count per simulation
        counts = rng.poisson(lam, size=n_sims)
        alive = np.arange(n_new)[None, :] < counts[:, None]
        S[:, M:] = np.where(alive, S[:, M:], -np.inf)

    order = np.argsort(-S, axis=1, kind="stable")
    top3 = np.take_along_axis(C, order[:, :3], axis=1)

    out = {}
    for k in (1, 2, 3):
        cnt = np.bincount(top3[:, k - 1], minlength=len(names))
        out[k] = {names[i]: c / n_sims for i, c in enumerate(cnt) if c}

    cn_mask = np.array([norm(c) in CHINESE for c in names])
    is_cn = cn_mask[C] & np.isfinite(S)
    big = np.where(is_cn, S, -np.inf)
    bestcn = np.argmax(big, axis=1)
    has = np.isfinite(big[np.arange(n_sims), bestcn])
    cnc = C[np.arange(n_sims), bestcn][has]
    cnt = np.bincount(cnc, minlength=len(names))
    tot = cnt.sum() or 1
    out["chinese1"] = {names[i]: c / tot for i, c in enumerate(cnt) if c}

    for k in out:
        out[k] = dict(sorted(out[k].items(), key=lambda z: -z[1]))
    return out


def to_legs(dist, legs):
    """Map a simulated company distribution onto a board's leg list, pooling into Other."""
    p = {l: 0.0 for l in legs}
    other = 0.0
    for c, v in dist.items():
        hit = next((l for l in legs if norm(l) == norm(c)), None)
        if hit:
            p[hit] += v
        else:
            other += v
    if "Other" in p:
        p["Other"] += other
    elif other:
        p["Other"] = other
    return p
