# temp-truncation

Exploit structural repricing lag in Polymarket's daily "Highest temperature in <city>"
bucket families. The resolution variable is a monotone running max of a free real-time
public feed, so intraday observations create mathematical zeros (legs below the running
max) and post-peak physics kills the upside tail — while forecast-anchored retail and
spread-thin market makers reprice slowly across ~1,400 parallel legs.

Born from `ideas/2026-07-23-temp-daily-max-truncation-lag.md` (market researcher's
first scan). Variants:

- [`runningmax/`](runningmax/) — running-max dead-leg detection + truncated diurnal
  model (trial, slot 1, started 2026-07-23).

Cross-variant lessons: (none yet)
