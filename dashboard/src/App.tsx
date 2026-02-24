import { useState } from 'react';
import { BillingPage } from './pages/Billing';
import { DashboardPage } from './pages/Dashboard';
import { InstancesPage } from './pages/Instances';
import './App.css';

type Page = 'dashboard' | 'instances' | 'billing';

// ---------------------------------------------------------------------------
// For development / demo, hard-code user ID 1.
// Replace with a real auth context when authentication is implemented.
// ---------------------------------------------------------------------------
const DEMO_USER_ID = 1;

function App() {
  const [page, setPage] = useState<Page>('dashboard');

  return (
    <>
      <nav style={{ display: 'flex', gap: '1rem', padding: '0.5rem 1rem', borderBottom: '1px solid #ccc' }}>
        <strong>Orsta</strong>
        <button onClick={() => setPage('dashboard')}>Dashboard</button>
        <button onClick={() => setPage('instances')}>Instances</button>
        <button onClick={() => setPage('billing')}>Billing</button>
      </nav>

      {page === 'dashboard' && <DashboardPage userId={DEMO_USER_ID} />}
      {page === 'instances' && <InstancesPage userId={DEMO_USER_ID} />}
      {page === 'billing' && <BillingPage userId={DEMO_USER_ID} />}
    </>
  );
}

export default App;
