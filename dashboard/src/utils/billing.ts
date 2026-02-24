// ---------------------------------------------------------------------------
// Billing utilities
// ---------------------------------------------------------------------------

/** Pricing constants (keep in sync with src/sql/billing.rs) */
export const HOURLY_RATE_CENTS = 48; // $0.48/hour
export const PROMOTION_DISCOUNT_PERCENT = 30;
export const PROMOTION_DURATION_SECS = 2 * 30 * 24 * 60 * 60; // ~2 months

/** Format an amount in cents as a USD dollar string, e.g. "$1.34" */
export function formatCents(cents: number): string {
  return `$${(cents / 100).toFixed(2)}`;
}

/**
 * Calculate the running charge (in cents) for an open billing window.
 *
 * @param startedAt  Unix timestamp (seconds) when the window opened.
 * @param userCreatedAt  Unix timestamp when the user registered.
 * @param nowSecs  Current Unix timestamp (seconds); defaults to Date.now()/1000.
 */
export function estimateOpenChargeCents(
  startedAt: number,
  userCreatedAt: number,
  nowSecs: number = Math.floor(Date.now() / 1000),
): number {
  const durationSecs = Math.max(0, nowSecs - startedAt);
  const hours = durationSecs / 3_600;
  const baseCents = Math.round(hours * HOURLY_RATE_CENTS);

  const withinPromotion =
    startedAt - userCreatedAt <= PROMOTION_DURATION_SECS;

  if (withinPromotion) {
    return Math.round(baseCents * (100 - PROMOTION_DISCOUNT_PERCENT) / 100);
  }
  return baseCents;
}

/** Convert a Unix timestamp (seconds) to a locale date/time string. */
export function unixToLocale(ts: number): string {
  return new Date(ts * 1_000).toLocaleString();
}
