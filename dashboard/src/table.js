/* orakel — sortable data tables.
 *
 * Served by the Worker at /table.js, included only by pages that need it
 * (render::table_sortable). Any <table class="data sortable"> becomes
 * sortable by clicking a header cell:
 *
 *   1st click  numeric column → descending (the biggest number is the answer)
 *              text column    → ascending
 *   2nd click  the other direction
 *   3rd click  back to the order the server rendered — every interaction has
 *              an exit, including this one.
 *
 * Values come from the cell's own text: thousands separators, %, ±, unicode
 * minus and a trailing "n = …" sub-line are stripped before parsing, so the
 * markup needs no sort attributes. Cells with no value ("—", empty) always
 * sort last, in both directions: a missing number is not a small one.
 *
 * Plain procedural code (CODING.md), no dependencies.
 */
(function () {
  "use strict";

  function cellValue(td) {
    if (!td) return { n: null, s: "" };
    /* Only the first line counts: dense cells carry a <span class="sub"> with
       the sample size under the number, which is context, not the value. */
    var main = td.querySelector(".sub") ? td.cloneNode(true) : td;
    if (main !== td) {
      var subs = main.querySelectorAll(".sub");
      for (var i = 0; i < subs.length; i++) subs[i].remove();
    }
    var text = (main.textContent || "").trim();
    if (!text || text === "—" || text === "-") return { n: null, s: "" };
    var cleaned = text
      .replace(/−/g, "-")
      .replace(/[,\s]/g, "")
      .replace(/[%$×]/g, "")
      .replace(/±.*$/, "");
    var n = parseFloat(cleaned);
    var numeric = !isNaN(n) && /^[-+]?[\d.]/.test(cleaned);
    return { n: numeric ? n : null, s: text.toLowerCase() };
  }

  function columnIsNumeric(rows, idx) {
    var seen = 0;
    for (var i = 0; i < rows.length; i++) {
      var v = cellValue(rows[i].cells[idx]);
      if (v.n !== null) seen++;
    }
    return seen >= Math.max(1, rows.length / 2);
  }

  function apply(table) {
    var tbody = table.tBodies[0];
    if (!tbody) return;
    var original = Array.prototype.slice.call(tbody.rows);
    var heads = table.tHead ? table.tHead.rows[0].cells : [];
    var state = { col: -1, dir: 0 }; // dir: 0 none, 1 asc, -1 desc

    function sortBy(idx) {
      var rows = Array.prototype.slice.call(tbody.rows);
      var numeric = columnIsNumeric(rows, idx);
      if (state.col !== idx) state.dir = numeric ? -1 : 1;
      else if (state.dir === (numeric ? -1 : 1)) state.dir = numeric ? 1 : -1;
      else state.dir = 0;
      state.col = idx;

      var order;
      if (state.dir === 0) {
        order = original.slice();
        state.col = -1;
      } else {
        var keyed = rows.map(function (r, i) {
          return { r: r, v: cellValue(r.cells[idx]), i: i };
        });
        keyed.sort(function (a, b) {
          var am = a.v.n === null && a.v.s === "";
          var bm = b.v.n === null && b.v.s === "";
          if (am !== bm) return am ? 1 : -1; // missing always last
          var d;
          if (numeric) d = (a.v.n === null ? -Infinity : a.v.n) - (b.v.n === null ? -Infinity : b.v.n);
          else d = a.v.s < b.v.s ? -1 : a.v.s > b.v.s ? 1 : 0;
          if (d === 0) return a.i - b.i; // stable
          return state.dir === 1 ? d : -d;
        });
        order = keyed.map(function (k) {
          return k.r;
        });
      }
      for (var i = 0; i < order.length; i++) tbody.appendChild(order[i]);
      for (var h = 0; h < heads.length; h++) {
        if (h === state.col && state.dir !== 0)
          heads[h].setAttribute("aria-sort", state.dir === 1 ? "ascending" : "descending");
        else heads[h].removeAttribute("aria-sort");
      }
    }

    for (var h = 0; h < heads.length; h++) (function (idx) {
      var th = heads[idx];
      th.tabIndex = 0;
      th.title = "Sort by this column";
      th.addEventListener("click", function () {
        sortBy(idx);
      });
      th.addEventListener("keydown", function (e) {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          sortBy(idx);
        }
      });
    })(h);
  }

  var tables = document.querySelectorAll("table.data.sortable");
  for (var i = 0; i < tables.length; i++) apply(tables[i]);
})();
