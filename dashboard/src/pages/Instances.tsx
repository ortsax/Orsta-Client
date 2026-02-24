import { InstanceTable } from '../components/instances/InstanceTable';

interface InstancesPageProps {
  userId: number;
}

export function InstancesPage({ userId }: InstancesPageProps) {
  return (
    <main>
      <h1>Manage Instances</h1>
      <InstanceTable userId={userId} />
    </main>
  );
}
