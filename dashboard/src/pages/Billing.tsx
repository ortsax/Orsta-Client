import { BillingTable } from '../components/billing/BillingTable';

interface BillingPageProps {
  userId: number;
}

export function BillingPage({ userId }: BillingPageProps) {
  return (
    <main>
      <h1>Billing</h1>
      <BillingTable userId={userId} />
    </main>
  );
}
