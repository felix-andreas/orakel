# Detecting wash / farmed volume

Polymarket's headline `volume` is a **share count, not dollars**, and on thin markets it
can be dominated by wash-trading or airdrop-farming. When it is, the quoted midpoint is
decorative and "the market is efficient" is void in both directions. Run this before
deferring to — or fading — a printed price. (Proven on poly's VKTX takeover market, where
**77% of notional was fake**.)

## The four tests

Pull the full tape (`data-api.polymarket.com/trades?market=<condition_id>&limit=500`,
paginate; usually only ~1–2k rows) and the live book, then:

1. **Share-volume vs dollar-notional.** Σ size×price = true notional. Headline `volume`
   >> notional ⇒ the headline counts shares. Then ask whether even the notional is real.
2. **Same-wallet matched pairs.** One wallet doing near-identical size on both sides at
   symmetric prices minutes apart = **self-crossing wash** — zero price information. Also
   check top-10-wallet share of notional (74% on VKTX; one wallet 47%).
3. **Multi-wallet churn rings at a pinned price.** Many wallets doing small ~equal
   bullish-then-bearish sweeps while the price sits pinned for days = **volume farming**.
4. **Real depth, not the volume stat.** Trust depth within 5c of mid and the
   market-order walk (a 1c quoted spread can hide a book where $1k slips 24c). Far-OTM
   ask walls at 0.9+ are exit orders, not opinion — exclude from imbalance metrics, but
   their *cancellation* is a signal (holders expecting resolution).

## What to trust instead of the mid

Signed taker flow over recent windows; share-weighted VWAPs (7/14/30/90d); the price path
as an event log (spikes vs baseline floor — on deadline markets a rising floor = rising
perceived hazard). Workable de-noise blend from VKTX:
`0.5×reversion-target + 0.25×30d-VWAP + 0.25×mid`, minus a haircut for an empty bid /
exit-heavy ask. In threshold families, use the sibling curve
([thin-market-price-read](thin-market-price-read.md)).
