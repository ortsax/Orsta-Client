import { BillingTable } from '../components/billing/BillingTable';
import { InstanceTable } from '../components/instances/InstanceTable';

interface DashboardPageProps {
  userId: number;
}

export function DashboardPage({ userId }: DashboardPageProps) {
  return (
    <main>
      <h1>Orsta Dashboard</h1>
      <InstanceTable userId={userId} />
      <hr />
      <BillingTable userId={userId} />
    </main>
  );
}
