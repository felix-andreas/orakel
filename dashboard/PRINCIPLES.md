# Dashboard principles

Binding rules for anyone touching this dashboard. Set by Felix, 2026-07-25.

## The rule

> **The goal is maximum bandwidth of information transport. No visual element that does
> not have a UX purpose.**

Every box, border, icon, colour, heading and line of chrome must justify itself by making
information *faster to absorb*. If it only makes the page look designed, delete it. A
dense, calm table beats a grid of decorated cards; a sentence in plain English beats a
metric nobody can interpret.

## What follows from it

- **No decoration.** No element exists to fill space, balance a layout, or look
  professional. Panels that hold one number are not panels.
- **Vertical space is the scarcest resource.** Headline boxes must be compact; the
  content people came for should be visible without scrolling.
- **Cards are the exception, not the container.** A card is a claim that its contents are
  a distinct object worth a border. Most content is not. Prefer **sections separated by
  space and a heading**; use a card only where a bordered box genuinely aids scanning.
- **Never nest a card in a card.** If you find yourself doing it, the hierarchy belongs in
  headings and spacing, not in borders. Nested boxes cost horizontal room, double the
  padding, and communicate nothing.
- **Density is a feature.** Wasted vertical space is wasted bandwidth. On mobile
  especially, card padding is the main thief — a list of five things should read as five
  lines, not five boxes.
- **Tables over cards** for anything with more than three attributes. Expandable rows
  beat separate pages for hierarchy (family → variants).
- **Plain English, no jargon — and self-contained.** Every strategy, run and decision
  needs a description a smart outsider can read **cold**, with no prior knowledge: what
  the thing is, what we do, and *why that should work*. An explanation that assumes you
  already know what a "leg" or a "ladder" or a "de-vigged mid" is has failed. Motivate the
  claim; state the catch. Internal vocabulary (annROLC, paired Brier,
  gate 2) may appear *next to* the plain sentence, never instead of it.
- **No walls of text.** Long documents get structure — summary line, then detail. If a
  page is a wall, it has failed regardless of the content's quality.
- **Contrast serves reading.** Body text is readable; section headers are readable.
  Never signal "inactive" by making text lighter — signal "active" by making it
  *stronger* (weight), so nothing on screen is hard to read.
- **Everything that looks navigable is navigable.** Breadcrumbs are always clickable at
  every level; slugs, families and variants are links wherever they appear.
- **No duplicate titles.** The breadcrumb is the title. One name per thing per screen.
- **Every interaction has an exit.** Zoomed a chart? There is a visible reset. Expanded a
  row? It collapses. State that persists (theme, sidebar) is remembered.
- **Numbers are comparable.** Tabular figures, consistent units, sample sizes beside
  every statistic, and never a number whose basis isn't stated.

## Test before shipping

Look at each page and ask: *what is the fastest question this page answers, and how many
seconds does it take?* Then remove whatever did not help answer it.
