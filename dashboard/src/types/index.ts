// ---------------------------------------------------------------------------
// Domain types – mirror the Rust/Diesel models
// ---------------------------------------------------------------------------

export interface User {
  id: number;
  username: string;
  email: string;
  /** Unix timestamp (seconds) when the user registered */
  created_at: number;
}

/** An active or inactive WhatsApp bot instance */
export interface Instance {
  id: number;
  user_id: number;
  /** ISO 3166-1 alpha-2 code (one of 180 WhatsApp-supported countries) */
  country_code: string;
  phone_number: string;
  /** 0 = inactive, 1 = active */
  active: number;
  /** Unix timestamp when the instance was created */
  created_at: number;
}

export interface NewInstance {
  user_id: number;
  country_code: string;
  phone_number: string;
}

/** One continuous active billing window for an instance */
export interface BillingRecord {
  id: number;
  instance_id: number;
  user_id: number;
  /** Unix timestamp when this window opened */
  started_at: number;
  /** Unix timestamp when this window closed, or null if still open */
  ended_at: number | null;
  /** Charge for this window in cents (0 while window is still open) */
  amount_cents: number;
}

export interface BillingSummary {
  user_id: number;
  records: BillingRecord[];
  /** Sum of all closed-window charges (cents) */
  total_cents: number;
}
