import React, { useState } from 'react';
import { useInstances } from '../../hooks/useInstances';
import type { NewInstance } from '../../types';

interface InstanceTableProps {
  userId: number;
}

export function InstanceTable({ userId }: InstanceTableProps) {
  const { instances, loading, error, activate, deactivate, create } =
    useInstances(userId);

  const [newPhone, setNewPhone] = useState('');
  const [newCountry, setNewCountry] = useState('');
  const [submitting, setSubmitting] = useState(false);

  async function handleCreate(e: React.FormEvent) {
    e.preventDefault();
    setSubmitting(true);
    try {
      const data: NewInstance = {
        user_id: userId,
        phone_number: newPhone,
        country_code: newCountry.toUpperCase(),
      };
      await create(data);
      setNewPhone('');
      setNewCountry('');
    } finally {
      setSubmitting(false);
    }
  }

  if (error) return <p className="error">Error: {error}</p>;

  return (
    <section>
      <h2>WhatsApp Bot Instances</h2>

      {/* Create form */}
      <form onSubmit={handleCreate} style={{ marginBottom: '1rem' }}>
        <input
          placeholder="Phone number"
          value={newPhone}
          onChange={(e) => setNewPhone(e.target.value)}
          required
        />
        <input
          placeholder="Country code (e.g. US)"
          maxLength={2}
          value={newCountry}
          onChange={(e) => setNewCountry(e.target.value)}
          required
        />
        <button type="submit" disabled={submitting}>
          {submitting ? 'Creating…' : 'Create Instance'}
        </button>
      </form>

      {/* Instance list */}
      {loading && instances.length === 0 ? (
        <p>Loading…</p>
      ) : (
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>Phone</th>
              <th>Country</th>
              <th>Status</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {instances.map((inst) => (
              <tr key={inst.id}>
                <td>{inst.id}</td>
                <td>{inst.phone_number}</td>
                <td>{inst.country_code}</td>
                <td>
                  <span
                    style={{ color: inst.active === 1 ? 'green' : 'grey' }}
                  >
                    {inst.active === 1 ? '● Active' : '○ Inactive'}
                  </span>
                </td>
                <td>
                  {inst.active === 0 ? (
                    <button onClick={() => activate(inst.id)}>Activate</button>
                  ) : (
                    <button onClick={() => deactivate(inst.id)}>
                      Deactivate
                    </button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}
