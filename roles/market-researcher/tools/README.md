# Market researcher tools

- `scan/` — landscape scanner. Pages Gamma `/events` (open+active), flattens nested
  markets to CSV, prints horizon/volume/tag summary.
  `cargo run --release -- <out.csv> [pages=15] [order=volume24hr]`
  (100 events/page; order = any Gamma sort key). CSV columns include condition_id,
  end_date, volume_num/24hr/1wk, liquidity, spread, bid/ask, yes_price, event tags.
