import { useBilling } from '../../hooks/useBilling';
import { formatCents, unixToLocale } from '../../utils/billing';

interface BillingTableProps {
  userId: number;
}

export function BillingTable({ userId }: BillingTableProps) {
  const { summary, loading, error } = useBilling(userId);

  if (error) return <p className="error">Error: {error}</p>;
  if (loading && !summary) return <p>Loading billing data…</p>;
  if (!summary) return null;

  return (
    <section>
      <h2>Billing</h2>
      <p>
        <strong>Total charged:</strong> {formatCents(summary.total_cents)}
      </p>
      <p style={{ fontSize: '0.85rem', color: '#555' }}>
        Rate: $0.48 / hour per active instance. New users receive a 30 %
        discount for the first 2 months.
      </p>

      <table>
        <thead>
          <tr>
            <th>Record ID</th>
            <th>Instance</th>
            <th>Started</th>
            <th>Ended</th>
            <th>Charge</th>
          </tr>
        </thead>
        <tbody>
          {summary.records.map((rec) => (
            <tr key={rec.id}>
              <td>{rec.id}</td>
              <td>{rec.instance_id}</td>
              <td>{unixToLocale(rec.started_at)}</td>
              <td>{rec.ended_at ? unixToLocale(rec.ended_at) : '(running)'}</td>
              <td>
                {rec.ended_at
                  ? formatCents(rec.amount_cents)
                  : <em>pending</em>}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}
