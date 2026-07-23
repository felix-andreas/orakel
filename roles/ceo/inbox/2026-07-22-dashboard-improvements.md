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

Chart architecture (decided with Felix 2026-07-22): **client-side rendering.** The
Worker serves JSON data endpoints; one hand-rolled, dependency-free JS chart framework
(single static file, ~400 lines, SVG-in-DOM, shadcn-token styled) renders all charts —
line + bar first — with local brush-zoom, tooltips, and container-resize. Rust stays the
compute layer: Worker-side JSON today, and later the backtest crate compiled to WASM
produces the same JSON shapes client-side for the same chart framework. Rationale: best
interactivity (no round-trips), one chart codebase forever.

## Reply
