'use client';

import { useRouter } from 'next/navigation';
import { useState } from 'react';
import { Nav } from '@/components/Nav';
import {
  PERMISSION_RESOURCES,
  registerAgent,
  type PermissionMode,
  type PermissionResource,
} from '@/lib/api';
import { getSession } from '@/lib/session';

function randomPublicKey(): number[] {
  const bytes = new Uint8Array(32);
  crypto.getRandomValues(bytes);
  return Array.from(bytes);
}

function toHex(bytes: number[]): string {
  return bytes.map((b) => b.toString(16).padStart(2, '0')).join('');
}

export default function RegisterPage() {
  const router = useRouter();
  const [version, setVersion] = useState('1.0.0');
  const [capabilities, setCapabilities] = useState('{}');
  const [publicKey, setPublicKey] = useState<number[]>(() => randomPublicKey());
  const [permissions, setPermissions] = useState<Record<PermissionResource, PermissionMode>>(
    () =>
      Object.fromEntries(PERMISSION_RESOURCES.map((r) => [r, 'Deny'])) as Record<
        PermissionResource,
        PermissionMode
      >
  );
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);

    let parsedCapabilities: unknown;
    try {
      parsedCapabilities = JSON.parse(capabilities);
    } catch {
      setError('Capabilities must be valid JSON.');
      return;
    }

    setSubmitting(true);
    try {
      const { ownerId } = await getSession();
      const res = await registerAgent({
        owner_id: ownerId,
        version,
        capabilities: parsedCapabilities,
        public_key: publicKey,
        permissions: PERMISSION_RESOURCES.map((r) => [r, permissions[r]]),
      });
      router.push(`/agents/${res.id}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <>
      <Nav current="register" />
      <div className="wrap" style={{ maxWidth: 640 }}>
        <h1 style={{ fontSize: 28, marginBottom: 'var(--sp-4)' }}>Register an Agent</h1>

        <form onSubmit={handleSubmit} className="card" style={{ display: 'flex', flexDirection: 'column', gap: 'var(--sp-4)' }}>
          <div>
            <label htmlFor="version">Version</label>
            <input
              id="version"
              value={version}
              onChange={(e) => setVersion(e.target.value)}
              required
              style={{ width: '100%' }}
            />
          </div>

          <div>
            <label htmlFor="capabilities">Capabilities (JSON)</label>
            <textarea
              id="capabilities"
              value={capabilities}
              onChange={(e) => setCapabilities(e.target.value)}
              rows={3}
              style={{ width: '100%', fontFamily: 'var(--font-mono)', fontSize: 13 }}
            />
          </div>

          <div>
            <label htmlFor="public-key">
              Public key (auto-generated — no wallet integration yet, see design doc)
            </label>
            <div style={{ display: 'flex', gap: 'var(--sp-2)' }}>
              <input
                id="public-key"
                readOnly
                value={toHex(publicKey)}
                style={{ flex: 1, fontFamily: 'var(--font-mono)', fontSize: 12 }}
              />
              <button
                type="button"
                className="btn btn-ghost"
                onClick={() => setPublicKey(randomPublicKey())}
              >
                Regenerate
              </button>
            </div>
          </div>

          <div>
            <label>Permissions — every resource defaults to Deny, opt in explicitly</label>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
              {PERMISSION_RESOURCES.map((resource) => (
                <div
                  key={resource}
                  style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                    background: 'var(--surface-raised)',
                    border: '1px solid var(--border)',
                    borderRadius: 'var(--radius-md)',
                    padding: '8px 12px',
                  }}
                >
                  <span style={{ fontSize: 13 }}>{resource}</span>
                  <select
                    value={permissions[resource]}
                    onChange={(e) =>
                      setPermissions((p) => ({
                        ...p,
                        [resource]: e.target.value as PermissionMode,
                      }))
                    }
                  >
                    <option value="Deny">deny</option>
                    <option value="Allow">allow</option>
                    <option value="Conditional">conditional</option>
                  </select>
                </div>
              ))}
            </div>
          </div>

          {error && <div style={{ color: 'var(--danger)', fontSize: 13 }}>{error}</div>}

          <button type="submit" className="btn btn-primary" disabled={submitting}>
            {submitting ? 'Registering...' : 'Register Agent'}
          </button>
        </form>
      </div>
    </>
  );
}
