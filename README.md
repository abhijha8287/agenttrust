# AgentTrust

A trust layer for autonomous AI agents: each agent gets an identity, explicit permissions and an audit trail, with trust scores anchored on the Solana blockchain.

## Why this exists
AI agents are being given access to files, terminals, code, money and email with little control over what they may do or record of what they did. This project explores a permission and audit layer that is checkable after the fact, not just a log on one server.

## What it does
- **Register an agent**: issues a DID and keypair, and sets allow, deny or conditional permissions for 8 resources (filesystem, terminal, GitHub, database, email, cloud, browser, wallet). Nothing is allowed by default.
- **Check an action**: the policy engine evaluates each request against the agent's stored permissions
- **Audit**: every permitted action goes to a two-judge LLM panel (safety and quality) whose verdict changes the agent's trust score
- **Anchor on-chain**: the new score and a SHA-256 hash of the audit are written to an Anchor program on Solana devnet, approved by a Squads 2-of-3 multisig
- A Next.js frontend shows agents, permissions, audits and on-chain records

## Project structure
```
agenttrust/
├── services/
│   ├── gateway/             Single public API entry
│   ├── identity-service/    DIDs, keys, permissions
│   ├── policy-engine/       Allow, deny, conditional checks
│   ├── agent-core/          Agent actions and routing
│   ├── audit-engine/        Two-judge LLM audits, trust scores
│   └── blockchain-service/  Solana writes via multisig
├── shared/                  Shared Rust types and auth
├── contracts/               Anchor program and Squads setup
├── frontend/                Next.js dashboard
├── design/                  UI design references
├── DESIGN.md                Design system notes
├── TODOS.md                 Open work items
├── docker-compose.yml       Four Postgres databases
└── Cargo.toml               Rust workspace
```

## Getting started
Requirements: Rust (stable), Docker, Node.js 18+, the Solana CLI and Anchor (for the blockchain service), a Gemini API key.
```bash
git clone https://github.com/abhijha8287/agenttrust.git
cd agenttrust
cp .env.example .env            # set INTERNAL_SERVICE_SECRET, CLIENT_JWT_SECRET, GEMINI_API_KEY, CONTRACTS_DIR
docker compose up -d            # four Postgres databases on ports 5433-5436
```
Then start each Rust service in its own terminal (`cargo run -p <service>`) with the environment shown in "Setup details" below, and start the frontend with `cd frontend && npm install && npm run dev`.

## Configuration
| Variable | Purpose |
|:--|:--|
| `INTERNAL_SERVICE_SECRET` | Shared secret between services |
| `CLIENT_JWT_SECRET` | Signs frontend session tokens |
| `GEMINI_API_KEY` | LLM judges in the audit engine |
| `CONTRACTS_DIR` | Absolute path to `contracts/` |
| `DATABASE_URL`, `*_SERVICE_URL`, `PORT` | Set per service (see below) |

## Tech stack
Rust (Axum, Tokio, SQLx), PostgreSQL, Gemini API, Solana, Anchor, Squads multisig, Next.js, TypeScript, Docker

## Architecture

```
frontend (Next.js, :3000)
   │
   ▼
gateway (:8080)  ── client JWT auth, CORS, proxies everything below
   │
   ├──▶ identity-service (:8081)   agents, permissions, trust_score  [owns identity-db :5433]
   │
   ├──▶ agent-core (:8083)         records execution attempts        [owns agent-core-db :5434]
   │         │
   │         ├──▶ policy-engine (:8082)   stateless allow/deny/conditional decision
   │         │         └──▶ calls identity-service for the agent's real permissions
   │         │
   │         └──▶ audit-engine (:8084)    2-judge LLM panel, audit hash, trust score update
   │                   │                  [owns audit-engine-db :5435]
   │                   ├──▶ calls identity-service to read/patch trust_score
   │                   └──▶ blockchain-service (:8085)   on-chain anchoring
   │                             │                        [owns blockchain-service-db :5436]
   │                             └──▶ shells out to contracts/squads/chain-worker.ts
   │                                       └──▶ Solana devnet, via a Squads 2-of-3 vault
   │
   └──▶ (dev-only) /dev/session   mints a JWT for local testing — stand-in for
                                   real wallet-signature auth, which isn't built yet
```

Every service owns its own Postgres database — no service reads another
service's tables directly. Cross-service calls always go through HTTP APIs,
gated by a shared `x-internal-secret` header (service-to-service auth,
distinct from the client-facing JWT the gateway issues).

## Setup details

### 1. Environment

```bash
cp .env.example .env
```

Fill in real values for `INTERNAL_SERVICE_SECRET`, `CLIENT_JWT_SECRET`
(any random string works for local dev), `GEMINI_API_KEY`, and
`CONTRACTS_DIR` (absolute path to this repo's `contracts/` folder).

### 2. Databases

```bash
docker compose up -d
```

Brings up four Postgres containers: `identity-db` (5433), `agent-core-db`
(5434), `audit-engine-db` (5435), `blockchain-service-db` (5436).

### 3. The Rust services

Each one runs migrations automatically on startup. Open a terminal per
service (or use a process manager) and run, from the repo root:

```bash
# identity-service
DATABASE_URL=postgres://identity:identity@localhost:5433/identity \
INTERNAL_SERVICE_SECRET=<your-secret> PORT=8081 \
cargo run -p identity-service

# policy-engine
IDENTITY_SERVICE_URL=http://localhost:8081 \
INTERNAL_SERVICE_SECRET=<your-secret> PORT=8082 \
cargo run -p policy-engine

# agent-core
DATABASE_URL=postgres://agent_core:agent_core@localhost:5434/agent_core \
POLICY_ENGINE_URL=http://localhost:8082 AUDIT_ENGINE_URL=http://localhost:8084 \
INTERNAL_SERVICE_SECRET=<your-secret> PORT=8083 \
cargo run -p agent-core

# audit-engine
DATABASE_URL=postgres://audit_engine:audit_engine@localhost:5435/audit_engine \
IDENTITY_SERVICE_URL=http://localhost:8081 BLOCKCHAIN_SERVICE_URL=http://localhost:8085 \
GEMINI_API_KEY=<your-key> INTERNAL_SERVICE_SECRET=<your-secret> PORT=8084 \
cargo run -p audit-engine

# blockchain-service (needs the Solana program deployed + multisig set up — see step 4)
DATABASE_URL=postgres://blockchain_service:blockchain_service@localhost:5436/blockchain_service \
IDENTITY_SERVICE_URL=http://localhost:8081 CONTRACTS_DIR=<absolute-path-to>/contracts \
INTERNAL_SERVICE_SECRET=<your-secret> PORT=8085 \
cargo run -p blockchain-service

# gateway (start last — proxies to everything above)
IDENTITY_SERVICE_URL=http://localhost:8081 AGENT_CORE_URL=http://localhost:8083 \
AUDIT_ENGINE_URL=http://localhost:8084 BLOCKCHAIN_SERVICE_URL=http://localhost:8085 \
INTERNAL_SERVICE_SECRET=<your-secret> CLIENT_JWT_SECRET=<your-jwt-secret> PORT=8080 \
cargo run -p gateway
```

Use the **same** `INTERNAL_SERVICE_SECRET` value everywhere — every service
checks it on every incoming request.

### 4. Solana devnet (one-time setup, for on-chain anchoring)

`blockchain-service` needs a deployed program and a live multisig before it
can anchor anything. Native Windows Solana tooling is unreliable, so this
runs through the Docker image built from `contracts/Dockerfile`:

```bash
cd contracts
docker build -t agenttrust-anchor-build .
```

Then, using that image (mounting `contracts/` and a persistent Solana
config directory):

1. Generate/fund a deployer keypair with devnet SOL — the public devnet
   faucet is aggressively rate-limited per IP, so use
   [faucet.solana.com](https://faucet.solana.com) in a browser if
   `solana airdrop` fails.
2. `anchor build` inside the container, then `solana program deploy
   target/deploy/agent_trust.so --program-id target/deploy/agent_trust-keypair.json
   --url devnet`.
3. Run `contracts/squads/setup-devnet.ts` (via `node
   node_modules/ts-node/dist/bin.js squads/setup-devnet.ts` from inside
   `contracts/`, with `TS_NODE_TRANSPILE_ONLY=true` set) — this creates the
   2-of-3 multisig on devnet and funds both hot-signer keys, writing
   `contracts/squads/multisig-config.json`.

This only needs to happen once per environment. After that,
`blockchain-service` reads `multisig-config.json` and the deployed program's
IDL directly — no further manual steps.

### 5. Frontend

```bash
cd frontend
cp .env.local.example .env.local
npm install
npm run dev
```

Open **http://localhost:3000**.

## Using it

- **`/register`** — register an agent: set its permissions per resource,
  submit, land on its profile page.
- **`/dashboard`** — every registered agent, with a live trust gauge.
- **`/agents/[id]`** — the full picture: permissions, an "Attempt an Action"
  form that runs the real policy → audit → anchor pipeline, a live
  execution feed, the audit timeline with real judge reasoning, and a
  "Verify on Solscan →" link once an action's hash lands on-chain
  (typically 10–15 seconds after the action, once the LLM judges and Squads
  approvals both complete).

## Known limitations

Tracked with full reasoning in [`TODOS.md`](./TODOS.md). The short version:

- The Squads multisig test in `contracts/tests/multisig_authority.ts` fails
  against a **local** test validator under Docker-on-Windows (a validator
  consensus/voting issue specific to that environment) — but the same flow
  is proven working against real devnet (see blockchain-service above), so
  the multisig integration itself is not in question.
- Trust scores and audit history only exist once an agent has actually had
  an action run against it — there's no backfill or simulated history.
- No real wallet-signature auth yet; the frontend uses a dev-only JWT
  session (`gateway`'s `/dev/session` endpoint) as a stand-in.
- Light mode, and CLI/SDK distribution, are both deliberately deferred —
  see `TODOS.md` for why.
