---
from: felix
to: ceo
date: 2026-07-22
status: open
subject: Dashboard improvements
---

Requested improvements, in Felix's words:

1. **Polish**: too many boxes, layout not clean — make it more shadcn-like.
2. **Mobile friendly**: in general; sidebar should become a proper burger menu on mobile.
3. **Favicon**: a funny, very simple sea-shell logo. Reference SVG (Huge Icons shell):

   ```svg
   <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24"><path fill="none" stroke="#888888" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M11.5 12A2.5 2.5 0 0 0 7 13.5c0 1.38 1 3.5 3.5 3.5c3 0 4.5-2.5 4.5-4.5c0-3-2-5.5-5.5-5.5S3 9.5 3 14c0 3.314 2.5 8 9 8c4.97 0 9-5.03 9-10S17 2 13 2a2 2 0 0 0-2 2v3"/></svg>
   ```

4. **Development page** with a charts sub-tab: two example charts (line + bar). Line chart
   must support selecting a horizontal region (vertical band) to zoom in; charts must be
   dynamic to page size. No chart library — hand-rolled.

Chart architecture (proposed, pending Felix's OK): pure-Rust SVG
renderer as a pure function `(data, dims, range) -> svg string`, executed in the Worker;
a small dependency-free vanilla JS glue file (~150 lines) for brush selection, resize,
and swapping in re-rendered SVG from the Worker. No WASM blob for now — but the pure-fn
design lets the same renderer compile into a client-side WASM bundle later (e.g. for the
interactive backtest page where a client-side backtest produces chart data locally).

## Reply
