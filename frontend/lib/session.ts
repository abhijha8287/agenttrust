// Dev-only identity: there's no wallet-signature auth built yet (see
// gateway/src/auth.rs), so each browser gets a random owner_id persisted in
// localStorage, and a JWT minted for it via the gateway's dev-only
// /dev/session endpoint. Real auth replaces this whole file, not extends it.

const GATEWAY_URL =
  process.env.NEXT_PUBLIC_GATEWAY_URL ?? 'http://localhost:8080';

const OWNER_ID_KEY = 'agenttrust_owner_id';
const TOKEN_KEY = 'agenttrust_dev_token';

function getOwnerId(): string {
  let ownerId = localStorage.getItem(OWNER_ID_KEY);
  if (!ownerId) {
    ownerId = crypto.randomUUID();
    localStorage.setItem(OWNER_ID_KEY, ownerId);
  }
  return ownerId;
}

export async function getSession(): Promise<{ ownerId: string; token: string }> {
  const ownerId = getOwnerId();
  const cached = localStorage.getItem(TOKEN_KEY);
  if (cached) {
    return { ownerId, token: cached };
  }

  const res = await fetch(`${GATEWAY_URL}/dev/session`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ sub: ownerId }),
  });
  if (!res.ok) {
    throw new Error(`failed to obtain dev session: ${res.status}`);
  }
  const { token } = (await res.json()) as { token: string };
  localStorage.setItem(TOKEN_KEY, token);
  return { ownerId, token };
}
