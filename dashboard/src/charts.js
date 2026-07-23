/* orakel charts — hand-rolled, dependency-free SVG chart framework.
 *
 * Served by the Worker at /charts.js like style.css: one static file, no
 * build step, no external libraries.
 *
 * API (see the Development page for live examples):
 *   Chart.line(el, data, opts)   data = { label, points: [{t, v}, ...] }
 *                                t = unix epoch milliseconds, sorted ascending
 *   Chart.bar(el, data, opts)    data = { label, bars: [{label, v}, ...] }
 *   opts (both, all optional):   { min, max }  — fix the y-domain
 *   Both return { render } (line also { reset }); call render() to redraw.
 *
 * Behavior:
 *   - Charts fill their container (give it a height in CSS; .chart does) and
 *     re-render on container resize (ResizeObserver) and light/dark flips.
 *   - Every color and font is read from the shadcn CSS custom properties on
 *     :root at render time — nothing hardcoded — so dark mode just works.
 *   - Line: hover tooltip on the nearest point; click-drag a horizontal
 *     brush (translucent band) to zoom into that x-range; double-click
 *     resets. Axes and ticks are recomputed on every render, so zooming
 *     keeps round tick values.
 *   - Bar: hover highlights the bar and shows a tooltip.
 *
 * This file will render predictions/scores/backtests later and is meant to
 * be extended by future agents: plain procedural code (CODING.md), data in →
 * SVG out. Add new chart types as new top-level functions on Chart.
 */
(function () {
  "use strict";

  var SVG = "http://www.w3.org/2000/svg";
  var DAY = 86400000;

  // ------------------------------------------------------------- helpers

  function el(name, attrs) {
    var node = document.createElementNS(SVG, name);
    for (var k in attrs) node.setAttribute(k, attrs[k]);
    return node;
  }

  function cssVar(name, fallback) {
    var v = getComputedStyle(document.documentElement)
      .getPropertyValue(name)
      .trim();
    return v || fallback;
  }

  /* Resolved at every render (not cached) so scheme flips pick up the new
     token values. Fallbacks only guard against a missing stylesheet. */
  function theme() {
    return {
      text: cssVar("--muted-foreground", "#71717a"),
      grid: cssVar("--border", "#e4e4e7"),
      bg: cssVar("--background", "#ffffff"),
      series1: cssVar("--chart-1", "#e76e50"),
      series2: cssVar("--chart-2", "#2a9d90"),
      brush: cssVar("--ring", "#a1a1aa"),
      mono: cssVar("--font-mono", "ui-monospace, monospace"),
    };
  }

  function escText(s) {
    return String(s).replace(/[&<>"']/g, function (c) {
      return { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c];
    });
  }

  // Heckbert's "nice numbers": snap a step to 1/2/5 × 10^k.
  function niceNum(x, round) {
    var exp = Math.floor(Math.log10(x));
    var f = x / Math.pow(10, exp);
    var nf;
    if (round) nf = f < 1.5 ? 1 : f < 3 ? 2 : f < 7 ? 5 : 10;
    else nf = f <= 1 ? 1 : f <= 2 ? 2 : f <= 5 ? 5 : 10;
    return nf * Math.pow(10, exp);
  }

  // ~count round tick values covering [min, max].
  function ticks(min, max, count) {
    if (min === max) {
      min -= 1;
      max += 1;
    }
    var step = niceNum((max - min) / Math.max(1, count), true);
    var out = [];
    for (var t = Math.ceil(min / step) * step; t <= max + step / 1e6; t += step)
      out.push(t);
    return out;
  }

  // Time ticks: aligned to UTC midnights when the span allows.
  function timeTicks(t0, t1, count) {
    if (t1 - t0 < 2 * DAY) return ticks(t0, t1, count);
    var stepDays = Math.max(1, Math.round((t1 - t0) / DAY / Math.max(1, count)));
    var out = [];
    for (var t = Math.ceil(t0 / DAY) * DAY; t <= t1; t += stepDays * DAY)
      out.push(t);
    return out;
  }

  function fmtTime(ms, spanMs) {
    var iso = new Date(Math.round(ms)).toISOString();
    // "MM-DD HH:MM" when zoomed tight, "MM-DD" otherwise
    return spanMs < 3 * DAY ? iso.slice(5, 16).replace("T", " ") : iso.slice(5, 10);
  }

  function fmtDate(ms) {
    return new Date(Math.round(ms)).toISOString().slice(0, 10);
  }

  function fmtNum(v) {
    if (Math.abs(v) >= 1000) return v.toFixed(0);
    if (Math.abs(v) >= 10) return String(+v.toFixed(1));
    return String(+v.toFixed(3));
  }

  /* Tooltip: one absolutely-positioned div per chart, styled by .chart-tip
     in style.css (tokens → dark mode for free). Coordinates are relative to
     the container, which .chart makes position:relative. */
  function makeTip(container) {
    var tip = document.createElement("div");
    tip.className = "chart-tip";
    tip.style.display = "none";
    container.appendChild(tip);
    return {
      show: function (x, y, html) {
        tip.innerHTML = html;
        tip.style.display = "block";
        var left = x + 12;
        if (left + tip.offsetWidth > container.clientWidth - 4)
          left = x - tip.offsetWidth - 12;
        tip.style.left = Math.max(4, left) + "px";
        tip.style.top = Math.max(4, y - tip.offsetHeight - 10) + "px";
      },
      hide: function () {
        tip.style.display = "none";
      },
    };
  }

  function tipHtml(label, value) {
    return (
      '<span class="t">' + escText(label) + '</span>' +
      '<span class="v">' + escText(value) + "</span>"
    );
  }

  // Re-render on container resize and on light/dark scheme changes.
  function watch(container, render) {
    if (typeof ResizeObserver !== "undefined")
      new ResizeObserver(function () {
        render();
      }).observe(container);
    var mq = window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)");
    if (mq && mq.addEventListener)
      mq.addEventListener("change", function () {
        render();
      });
  }

  function yAxis(svg, th, yTicks, y, padLeft, innerW) {
    yTicks.forEach(function (tv) {
      svg.appendChild(
        el("line", {
          x1: padLeft, x2: padLeft + innerW,
          y1: y(tv), y2: y(tv),
          stroke: th.grid, "stroke-width": 1,
        })
      );
      var lbl = el("text", {
        x: padLeft - 8, y: y(tv) + 3,
        "text-anchor": "end", fill: th.text,
        "font-size": 10, "font-family": th.mono,
      });
      lbl.textContent = fmtNum(tv);
      svg.appendChild(lbl);
    });
  }

  // ----------------------------------------------------------- line chart

  function line(container, data, opts) {
    opts = opts || {};
    var pts = data.points || [];
    if (pts.length < 2) {
      container.textContent = "no data";
      return null;
    }
    var full = [pts[0].t, pts[pts.length - 1].t];
    var domain = full.slice(); // current zoom range [t0, t1]
    var tip = makeTip(container);
    var svg = null;

    function visible() {
      var vis = pts.filter(function (p) {
        return p.t >= domain[0] && p.t <= domain[1];
      });
      return vis.length >= 2 ? vis : pts;
    }

    function render() {
      var th = theme();
      var W = container.clientWidth || 600;
      var H = container.clientHeight || 320;
      if (svg) svg.remove();
      svg = el("svg", { width: W, height: H, viewBox: "0 0 " + W + " " + H });

      var pad = { top: 12, right: 12, bottom: 26, left: 44 };
      var iw = W - pad.left - pad.right;
      var ih = H - pad.top - pad.bottom;
      var vis = visible();
      var t0 = domain[0], t1 = domain[1], span = t1 - t0;

      var vmin = Infinity, vmax = -Infinity;
      vis.forEach(function (p) {
        vmin = Math.min(vmin, p.v);
        vmax = Math.max(vmax, p.v);
      });
      var vpad = (vmax - vmin || 1) * 0.1;
      vmin = opts.min !== undefined ? opts.min : vmin - vpad;
      vmax = opts.max !== undefined ? opts.max : vmax + vpad;

      function x(t) { return pad.left + ((t - t0) / span) * iw; }
      function y(v) { return pad.top + ih - ((v - vmin) / (vmax - vmin)) * ih; }

      // axes (ticks recomputed per render → stay round while zooming)
      yAxis(svg, th, ticks(vmin, vmax, Math.max(2, Math.round(ih / 45))), y, pad.left, iw);
      timeTicks(t0, t1, Math.max(2, Math.round(iw / 80))).forEach(function (tt) {
        var lbl = el("text", {
          x: x(tt), y: pad.top + ih + 16,
          "text-anchor": "middle", fill: th.text,
          "font-size": 10, "font-family": th.mono,
        });
        lbl.textContent = fmtTime(tt, span);
        svg.appendChild(lbl);
      });
      svg.appendChild(el("line", {
        x1: pad.left, x2: pad.left + iw,
        y1: pad.top + ih, y2: pad.top + ih,
        stroke: th.grid, "stroke-width": 1,
      }));

      // area + line
      var d = "";
      vis.forEach(function (p, i) {
        d += (i ? "L" : "M") + x(p.t).toFixed(1) + " " + y(p.v).toFixed(1);
      });
      var base = (pad.top + ih).toFixed(1);
      var area = d +
        "L" + x(vis[vis.length - 1].t).toFixed(1) + " " + base +
        "L" + x(vis[0].t).toFixed(1) + " " + base + "Z";
      svg.appendChild(el("path", { d: area, fill: th.series1, "fill-opacity": 0.08 }));
      svg.appendChild(el("path", {
        d: d, fill: "none", stroke: th.series1,
        "stroke-width": 1.8, "stroke-linejoin": "round", "stroke-linecap": "round",
      }));

      // hover marker, brush band, interaction overlay
      var marker = el("circle", {
        r: 3.5, fill: th.series1,
        stroke: th.bg, "stroke-width": 1.5, display: "none",
      });
      var band = el("rect", {
        y: pad.top, height: ih,
        fill: th.brush, "fill-opacity": 0.18, display: "none",
      });
      var overlay = el("rect", {
        x: pad.left, y: pad.top, width: iw, height: ih,
        fill: "transparent", cursor: "crosshair",
      });
      svg.appendChild(marker);
      svg.appendChild(band);
      svg.appendChild(overlay);

      function evtX(e) {
        var r = svg.getBoundingClientRect();
        return Math.min(pad.left + iw, Math.max(pad.left, e.clientX - r.left));
      }
      function pxToT(px) { return t0 + ((px - pad.left) / iw) * span; }
      function nearest(t) {
        var best = vis[0], dist = Infinity;
        vis.forEach(function (p) {
          var dd = Math.abs(p.t - t);
          if (dd < dist) { dist = dd; best = p; }
        });
        return best;
      }
      function containerXY(pxInSvg, pyInSvg) {
        var r = svg.getBoundingClientRect();
        var c = container.getBoundingClientRect();
        return [pxInSvg + r.left - c.left, pyInSvg + r.top - c.top];
      }

      var dragFrom = null;

      overlay.addEventListener("mousemove", function (e) {
        var px = evtX(e);
        if (dragFrom !== null) {
          band.setAttribute("x", Math.min(dragFrom, px));
          band.setAttribute("width", Math.abs(px - dragFrom));
          band.removeAttribute("display");
          return;
        }
        var p = nearest(pxToT(px));
        marker.setAttribute("cx", x(p.t));
        marker.setAttribute("cy", y(p.v));
        marker.removeAttribute("display");
        var xy = containerXY(x(p.t), y(p.v));
        tip.show(xy[0], xy[1], tipHtml(fmtDate(p.t), fmtNum(p.v)));
      });
      overlay.addEventListener("mousedown", function (e) {
        e.preventDefault();
        dragFrom = evtX(e);
        marker.setAttribute("display", "none");
        tip.hide();
      });
      overlay.addEventListener("mouseup", function (e) {
        if (dragFrom === null) return;
        var a = Math.min(dragFrom, evtX(e));
        var b = Math.max(dragFrom, evtX(e));
        dragFrom = null;
        band.setAttribute("display", "none");
        if (b - a < 8) return; // a click, not a brush
        domain = [pxToT(a), pxToT(b)];
        render();
      });
      overlay.addEventListener("mouseleave", function () {
        dragFrom = null;
        band.setAttribute("display", "none");
        marker.setAttribute("display", "none");
        tip.hide();
      });
      overlay.addEventListener("dblclick", function () {
        domain = full.slice();
        render();
      });

      container.appendChild(svg);
    }

    watch(container, render);
    render();
    return {
      render: render,
      reset: function () {
        domain = full.slice();
        render();
      },
    };
  }

  // ------------------------------------------------------------ bar chart

  function bar(container, data, opts) {
    opts = opts || {};
    var bars = data.bars || [];
    if (!bars.length) {
      container.textContent = "no data";
      return null;
    }
    var tip = makeTip(container);
    var svg = null;

    function render() {
      var th = theme();
      var W = container.clientWidth || 600;
      var H = container.clientHeight || 320;
      if (svg) svg.remove();
      svg = el("svg", { width: W, height: H, viewBox: "0 0 " + W + " " + H });

      var pad = { top: 12, right: 12, bottom: 26, left: 44 };
      var iw = W - pad.left - pad.right;
      var bandW = iw / bars.length;
      var rotated = bandW < 90; // long labels: slant them
      if (rotated) pad.bottom = 64;
      var ih = H - pad.top - pad.bottom;

      var vmax = 0;
      bars.forEach(function (b) { vmax = Math.max(vmax, b.v); });
      var ymax = opts.max !== undefined ? opts.max : (vmax || 1) * 1.15;
      function y(v) { return pad.top + ih - (v / ymax) * ih; }

      yAxis(svg, th, ticks(0, ymax, Math.max(2, Math.round(ih / 45))), y, pad.left, iw);
      svg.appendChild(el("line", {
        x1: pad.left, x2: pad.left + iw,
        y1: pad.top + ih, y2: pad.top + ih,
        stroke: th.grid, "stroke-width": 1,
      }));

      var bw = Math.min(bandW * 0.65, 64);
      bars.forEach(function (b, i) {
        var cx = pad.left + bandW * (i + 0.5);
        var rect = el("rect", {
          x: cx - bw / 2, y: y(b.v),
          width: bw, height: Math.max(0, pad.top + ih - y(b.v)),
          rx: 3, fill: th.series2, "fill-opacity": 0.85,
        });
        rect.addEventListener("mouseenter", function () {
          rect.setAttribute("fill-opacity", 1);
          var r = svg.getBoundingClientRect();
          var c = container.getBoundingClientRect();
          tip.show(
            cx + r.left - c.left,
            y(b.v) + r.top - c.top,
            tipHtml(b.label, fmtNum(b.v))
          );
        });
        rect.addEventListener("mouseleave", function () {
          rect.setAttribute("fill-opacity", 0.85);
          tip.hide();
        });
        svg.appendChild(rect);

        var ly = pad.top + ih + (rotated ? 12 : 16);
        var lbl = el("text", {
          x: cx, y: ly,
          "text-anchor": rotated ? "end" : "middle",
          fill: th.text, "font-size": 10, "font-family": th.mono,
        });
        if (rotated)
          lbl.setAttribute("transform", "rotate(-35 " + cx + " " + ly + ")");
        lbl.textContent = b.label;
        svg.appendChild(lbl);
      });

      container.appendChild(svg);
    }

    watch(container, render);
    render();
    return { render: render };
  }

  window.Chart = { line: line, bar: bar };
})();
