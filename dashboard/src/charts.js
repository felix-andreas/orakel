/* orakel charts — hand-rolled, dependency-free SVG chart framework.
 *
 * Served by the Worker at /charts.js like style.css: one static file, no build
 * step, no external libraries.
 *
 * API
 *   Chart.line(el, data, opts)
 *     data = { label, points: [{t, v, label?}] }                  one series
 *          | { label, series: [{ label, points, mode, color }] }  many series
 *     mode  = "line" (default) | "dots"
 *     color = index into the chart palette; defaults to the series position,
 *             pin it when a series may be absent at render time
 *   Chart.bar(el, data, opts)
 *     data = { label, bars: [{label, v, tone}] }   tone: "" | "ok" | "bad"
 *   opts (both, optional):
 *     { min, max }      fix the y-domain
 *     { x: "time" }     default — t is unix epoch ms, ticks snap to UTC days
 *     { x: "index" }    t is an ordinal position; ticks are plain numbers
 *     { yPrecision }    decimals in tick/tooltip numbers (default: auto)
 *   Both return { render }; line also { reset }.
 *
 * Behaviour
 *   - Charts fill their container (give it a height in CSS; .chart does) and
 *     re-render on container resize, on system light/dark flips AND on the
 *     dashboard's own theme toggle (data-theme on <html>, MutationObserver).
 *   - Every colour and font is read from the CSS custom properties at render
 *     time — nothing hardcoded — so both themes just work.
 *   - Line: hover tooltip on the nearest point across all series; click-drag a
 *     horizontal brush to zoom. A "Reset zoom" button appears while zoomed
 *     (double-click still works, but a hidden gesture is not an exit).
 *   - Bar: hover highlights; supports negative values with a zero baseline.
 *
 * Plain procedural code (CODING.md), data in → SVG out. Add new chart types as
 * new top-level functions on Chart.
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

  /* Resolved at every render (not cached) so theme flips pick up new values. */
  function theme() {
    return {
      text: cssVar("--fg-subtle", "#9ca3af"),
      grid: cssVar("--border", "#e7e7ea"),
      bg: cssVar("--panel", "#ffffff"),
      brush: cssVar("--ring", "#a1a1aa"),
      ok: cssVar("--ok", "#15803d"),
      bad: cssVar("--bad", "#b91c1c"),
      mono: cssVar("--font-mono", "ui-monospace, monospace"),
      series: [
        cssVar("--chart-1", "#2563eb"),
        cssVar("--chart-2", "#0f766e"),
        cssVar("--chart-3", "#b45309"),
        cssVar("--chart-4", "#7c3aed"),
        cssVar("--chart-5", "#be123c"),
        cssVar("--chart-6", "#4d7c0f"),
        cssVar("--chart-7", "#0369a1"),
        cssVar("--chart-8", "#a21caf"),
      ],
    };
  }

  function escText(s) {
    return String(s).replace(/[&<>"']/g, function (c) {
      return { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c];
    });
  }

  // Heckbert's "nice numbers": snap a step to 1/2/5 × 10^k.
  function niceNum(x, round) {
    if (!(x > 0)) return 1;
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
    for (var t = Math.ceil(t0 / DAY) * DAY; t <= t1; t += stepDays * DAY) out.push(t);
    return out;
  }

  function fmtTime(ms, spanMs) {
    var iso = new Date(Math.round(ms)).toISOString();
    return spanMs < 3 * DAY ? iso.slice(5, 16).replace("T", " ") : iso.slice(5, 10);
  }

  function fmtDateTime(ms) {
    return new Date(Math.round(ms)).toISOString().slice(0, 16).replace("T", " ");
  }

  function fmtNum(v, prec) {
    if (prec !== undefined) return v.toFixed(prec);
    if (Math.abs(v) >= 1000) return v.toFixed(0);
    if (Math.abs(v) >= 10) return String(+v.toFixed(1));
    if (Math.abs(v) >= 1) return String(+v.toFixed(2));
    return String(+v.toFixed(4));
  }

  /* Tooltip: one absolutely-positioned div per chart, styled by .chart-tip. */
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
      '<span class="t">' + escText(label) + "</span>" +
      '<span class="v">' + escText(value) + "</span>"
    );
  }

  /* Re-render on resize, on system scheme change, and on the theme toggle. */
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
    if (typeof MutationObserver !== "undefined")
      new MutationObserver(function () {
        render();
      }).observe(document.documentElement, { attributes: true, attributeFilter: ["data-theme"] });
  }

  function yAxis(svg, th, yTicks, y, padLeft, innerW, prec) {
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
      lbl.textContent = fmtNum(tv, prec);
      svg.appendChild(lbl);
    });
  }

  // ----------------------------------------------------------- line chart

  function line(container, data, opts) {
    opts = opts || {};
    var isIndex = opts.x === "index";
    // Normalise single-series input to the multi-series shape.
    var series = data.series || [{ label: data.label || "", points: data.points || [] }];
    /* Colour is pinned to the series' declared index (or its position before
       filtering), so dropping an empty series never recolours the rest. */
    series.forEach(function (s, i) {
      s._c = s.color !== undefined ? s.color : i;
    });
    series = series.filter(function (s) {
      return s.points && s.points.length;
    });
    var total = series.reduce(function (n, s) {
      return n + s.points.length;
    }, 0);
    if (!series.length || total < 2) {
      container.textContent = "not enough data to plot";
      return null;
    }

    var allT = [];
    series.forEach(function (s) {
      s.points.forEach(function (p) {
        allT.push(p.t);
      });
    });
    var full = [Math.min.apply(null, allT), Math.max.apply(null, allT)];
    if (full[0] === full[1]) {
      full = [full[0] - 1, full[1] + 1];
    }
    var domain = full.slice();
    var tip = makeTip(container);
    var svg = null;

    /* Every interaction needs a visible exit: the brush zoom gets a Reset
       button that appears only while zoomed. Double-click still works. */
    var reset = document.createElement("button");
    reset.type = "button";
    reset.className = "chart-reset";
    reset.textContent = "Reset zoom";
    reset.style.display = "none";
    reset.addEventListener("click", function () {
      domain = full.slice();
      render();
    });
    container.appendChild(reset);

    function visible(s) {
      var vis = s.points.filter(function (p) {
        return p.t >= domain[0] && p.t <= domain[1];
      });
      return vis;
    }

    function render() {
      var th = theme();
      var W = container.clientWidth || 600;
      var H = container.clientHeight || 320;
      if (svg) svg.remove();
      svg = el("svg", { width: W, height: H, viewBox: "0 0 " + W + " " + H });

      var pad = { top: 10, right: 12, bottom: 24, left: 48 };
      var iw = W - pad.left - pad.right;
      var ih = H - pad.top - pad.bottom;
      var t0 = domain[0], t1 = domain[1], span = t1 - t0 || 1;

      var vis = series.map(visible);
      var vmin = Infinity, vmax = -Infinity;
      vis.forEach(function (pts) {
        pts.forEach(function (p) {
          vmin = Math.min(vmin, p.v);
          vmax = Math.max(vmax, p.v);
        });
      });
      if (!isFinite(vmin)) {
        vmin = 0;
        vmax = 1;
      }
      var vpad = (vmax - vmin || Math.abs(vmax) || 1) * 0.12;
      vmin = opts.min !== undefined ? opts.min : vmin - vpad;
      vmax = opts.max !== undefined ? opts.max : vmax + vpad;
      if (vmin === vmax) vmax = vmin + 1;

      function x(t) { return pad.left + ((t - t0) / span) * iw; }
      function y(v) { return pad.top + ih - ((v - vmin) / (vmax - vmin)) * ih; }

      yAxis(svg, th, ticks(vmin, vmax, Math.max(2, Math.round(ih / 44))), y, pad.left, iw, opts.yPrecision);

      var xTicks = isIndex
        ? ticks(t0, t1, Math.max(2, Math.round(iw / 90)))
        : timeTicks(t0, t1, Math.max(2, Math.round(iw / 90)));
      xTicks.forEach(function (tt) {
        if (tt < t0 || tt > t1) return;
        var lbl = el("text", {
          x: x(tt), y: pad.top + ih + 15,
          "text-anchor": "middle", fill: th.text,
          "font-size": 10, "font-family": th.mono,
        });
        lbl.textContent = isIndex ? String(Math.round(tt)) : fmtTime(tt, span);
        svg.appendChild(lbl);
      });
      svg.appendChild(el("line", {
        x1: pad.left, x2: pad.left + iw,
        y1: pad.top + ih, y2: pad.top + ih,
        stroke: th.grid, "stroke-width": 1,
      }));

      // series
      vis.forEach(function (pts, si) {
        if (!pts.length) return;
        var colour = th.series[series[si]._c % th.series.length];
        var mode = series[si].mode || "line";
        if (mode === "dots") {
          pts.forEach(function (p) {
            svg.appendChild(el("circle", {
              cx: x(p.t).toFixed(1), cy: y(p.v).toFixed(1), r: 2.6,
              fill: colour, "fill-opacity": 0.85,
            }));
          });
          return;
        }
        var d = "";
        pts.forEach(function (p, i) {
          d += (i ? "L" : "M") + x(p.t).toFixed(1) + " " + y(p.v).toFixed(1);
        });
        if (si === 0 && series.length === 1) {
          var base = (pad.top + ih).toFixed(1);
          svg.appendChild(el("path", {
            d: d + "L" + x(pts[pts.length - 1].t).toFixed(1) + " " + base +
               "L" + x(pts[0].t).toFixed(1) + " " + base + "Z",
            fill: colour, "fill-opacity": 0.07,
          }));
        }
        svg.appendChild(el("path", {
          d: d, fill: "none", stroke: colour,
          "stroke-width": 1.8, "stroke-linejoin": "round", "stroke-linecap": "round",
        }));
      });

      // interaction layer
      var marker = el("circle", {
        r: 3.5, fill: th.series[0], stroke: th.bg, "stroke-width": 1.5, display: "none",
      });
      var band = el("rect", {
        y: pad.top, height: ih, fill: th.brush, "fill-opacity": 0.16, display: "none",
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
        var best = null, dist = Infinity, bestSeries = 0;
        vis.forEach(function (pts, si) {
          pts.forEach(function (p) {
            var dd = Math.abs(p.t - t);
            if (dd < dist) { dist = dd; best = p; bestSeries = si; }
          });
        });
        return best ? { p: best, si: bestSeries } : null;
      }
      function containerXY(px, py) {
        var r = svg.getBoundingClientRect();
        var c = container.getBoundingClientRect();
        return [px + r.left - c.left, py + r.top - c.top];
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
        var hit = nearest(pxToT(px));
        if (!hit) return;
        marker.setAttribute("cx", x(hit.p.t));
        marker.setAttribute("cy", y(hit.p.v));
        marker.setAttribute("fill", th.series[series[hit.si]._c % th.series.length]);
        marker.removeAttribute("display");
        var xy = containerXY(x(hit.p.t), y(hit.p.v));
        var head = hit.p.label
          ? hit.p.label
          : isIndex
          ? "#" + Math.round(hit.p.t)
          : fmtDateTime(hit.p.t);
        var name = series[hit.si].label;
        tip.show(xy[0], xy[1], tipHtml(head + (name ? " · " + name : ""), fmtNum(hit.p.v, opts.yPrecision)));
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
      var zoomed = domain[0] > full[0] || domain[1] < full[1];
      reset.style.display = zoomed ? "block" : "none";
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

      var pad = { top: 10, right: 12, bottom: 24, left: 52 };
      var iw = W - pad.left - pad.right;
      var bandW = iw / bars.length;
      var rotated = bandW < 56;
      if (rotated) pad.bottom = 58;
      var ih = H - pad.top - pad.bottom;

      var vmax = -Infinity, vmin = Infinity;
      bars.forEach(function (b) {
        vmax = Math.max(vmax, b.v);
        vmin = Math.min(vmin, b.v);
      });
      var top = opts.max !== undefined ? opts.max : vmax > 0 ? vmax * 1.15 : 0;
      var bottom = opts.min !== undefined ? opts.min : vmin < 0 ? vmin * 1.15 : 0;
      if (top === bottom) top = bottom + 1;
      function y(v) { return pad.top + ih - ((v - bottom) / (top - bottom)) * ih; }

      yAxis(svg, th, ticks(bottom, top, Math.max(2, Math.round(ih / 44))), y, pad.left, iw, opts.yPrecision);
      svg.appendChild(el("line", {
        x1: pad.left, x2: pad.left + iw,
        y1: y(0), y2: y(0),
        stroke: th.grid, "stroke-width": 1,
      }));

      var bw = Math.min(bandW * 0.62, 40);
      bars.forEach(function (b, i) {
        var cx = pad.left + bandW * (i + 0.5);
        var colour = b.tone === "ok" ? th.ok : b.tone === "bad" ? th.bad : th.series[0];
        var yTop = Math.min(y(b.v), y(0));
        var h = Math.max(1, Math.abs(y(b.v) - y(0)));
        var rect = el("rect", {
          x: cx - bw / 2, y: yTop, width: bw, height: h,
          rx: 2, fill: colour, "fill-opacity": 0.85,
        });
        rect.addEventListener("mouseenter", function () {
          rect.setAttribute("fill-opacity", 1);
          var r = svg.getBoundingClientRect();
          var c = container.getBoundingClientRect();
          tip.show(cx + r.left - c.left, yTop + r.top - c.top,
            tipHtml(b.label, fmtNum(b.v, opts.yPrecision)));
        });
        rect.addEventListener("mouseleave", function () {
          rect.setAttribute("fill-opacity", 0.85);
          tip.hide();
        });
        svg.appendChild(rect);

        if (!rotated || bandW > 14) {
          var ly = pad.top + ih + (rotated ? 12 : 15);
          var lbl = el("text", {
            x: cx, y: ly,
            "text-anchor": rotated ? "end" : "middle",
            fill: th.text, "font-size": 10, "font-family": th.mono,
          });
          if (rotated) lbl.setAttribute("transform", "rotate(-40 " + cx + " " + ly + ")");
          lbl.textContent = b.label;
          svg.appendChild(lbl);
        }
      });

      container.appendChild(svg);
    }

    watch(container, render);
    render();
    return { render: render };
  }

  window.Chart = { line: line, bar: bar };
})();
