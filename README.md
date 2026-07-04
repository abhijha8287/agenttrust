# AgentTrust

**The decentralized trust layer for autonomous AI agents.**

Every agent gets a verifiable identity, explicit per-resource permissions,
and an audit trail anchored on Solana. Nothing here is a mock — every piece
described below is a real, working system: real Postgres databases per
service, a real 2-judge LLM panel, a real Anchor program deployed to Solana
devnet, and a real Squads 2-of-3 multisig signing real transactions.

## What it does, end to end

1. **Register an agent** — gets a DID, a keypair, and explicit allow /
   deny / conditional permissions across 8 resources (filesystem, terminal,
   GitHub, database, email, cloud, browser, wallet). Nothing is allowed by
   default.
2. **Attempt an action** — the agent asks to do something against one of
   those resources. `policy-engine` evaluates it against the agent's actual
   stored permissions and returns allow / deny / conditional.
3. **Get audited** — every non-denied action is sent to a real 2-judge LLM
   panel (a Safety Judge and a Quality Judge, both genuine model calls) that
   reach a verdict and adjust the agent's trust score.
4. **Get anchored on-chain** — the new trust score and a SHA-256 hash of the
   audit are written to a Solana program on devnet, authorized by a Squads
   2-of-3 multisig where two automated "hot signer" keys approve
   automatically (no human in the loop for routine writes) and a third,
   cold key exists only for recovery.

All of it is visible in a real Next.js frontend, not just via API calls.

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

## Repository structure

```
AgentTrust/
├── shared/                     Rust crate shared by every service:
│                                error types, internal-auth middleware, DTOs
│
├── services/
│   ├── identity-service/        Agent registry: register, list, get, update
│   │                            trust score. Owns the agents + permissions tables.
│   ├── gateway/                 The only service the frontend talks to.
│   │                            Client JWT auth, CORS, reverse-proxies to
│   │                            every service below.
│   ├── policy-engine/           Stateless. Given an agent + resource, fetches
│   │                            that agent's real permissions from
│   │                            identity-service and returns a decision.
│   ├── agent-core/               Records every execution attempt and its
│   │                            policy decision. Triggers an audit in the
│   │                            background for anything not denied.
│   ├── audit-engine/            Runs the real 2-judge LLM panel (Gemini),
│   │                            computes the audit hash, updates trust_score,
│   │                            triggers on-chain anchoring in the background.
│   └── blockchain-service/      Registers agents on-chain (lazily, on first
│                                anchor) and submits the Squads-mediated vault
│                                transaction that writes the score + hash to
│                                the Solana program.
│
├── contracts/                  The Solana side.
│   ├── programs/agent_trust/    The Anchor program itself (register_agent,
│   │                            update_trust_score, record_audit_hash).
│   ├── squads/                  Everything Squads-related:
│   │   ├── setup-multisig.ts     Creates the 2-of-3 multisig (reusable for
│   │   │                         local-validator tests or devnet).
│   │   ├── setup-devnet.ts       Runs it against real devnet + funds the two
│   │   │                         hot-signer keys via transfer (not airdrop).
│   │   ├── chain-worker.ts       blockchain-service's actual on-chain client
│   │   │                         — Squads v4 has no Rust SDK, so this is
│   │   │                         invoked as a subprocess from Rust.
│   │   └── multisig-config.json  Generated, gitignored. Holds the real
│   │                             hot-signer keypairs — never commit this.
│   ├── tests/                   Anchor test suite (single-authority +
│   │                            Squads-multisig-authority paths).
│   └── Dockerfile                Solana CLI 1.18.17 + Anchor 0.30.1, pinned
│                                 deliberately (see comments in Cargo.toml
│                                 for why — newer toolchains break the build).
│
├── frontend/                   Next.js app (App Router, TypeScript).
│   ├── app/                     landing (`/`), dashboard, register, and the
│   │                            agent profile page (trust gauge, permissions,
│   │                            live execution + audit + on-chain history).
│   ├── components/               TrustGauge (the signature radial gauge) and
│   │                            the shared site nav.
│   └── lib/                      API client + a dev-only per-browser session
│                                 helper (localStorage UUID + JWT).
│
├── design/reference/           Static HTML mockups from the design phase —
│                                visual reference only, not wired to anything.
│
├── docker-compose.yml           One Postgres container per service.
├── DESIGN.md                    The design system — read this before any UI change.
├── TODOS.md                     Known gaps and deferred work, with reasoning.
└── .env.example                  Template for every env var every service needs.
```

## Prerequisites

- **Rust** (native toolchain — `cargo --version`, any recent stable release)
- **Node.js** 20+ and **npm**
- **Docker Desktop** (only needed for Postgres containers and for building/
  deploying the Solana program — not needed to run the Rust services or the
  frontend, which run natively)
- A **Gemini API key** (audit-engine's judge panel) — get one at
  [aistudio.google.com/apikey](https://aistudio.google.com/apikey)
- **Never paste API keys directly into a chat/terminal session that logs
  transcripts.** Put them in `.env` (already gitignored) instead.

## Running it locally

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

## Design system

Everything visual is governed by [`DESIGN.md`](./DESIGN.md) — read it before
touching any styling. The short version: Technical Restraint (Linear/Stripe/
Vercel-inspired dark dev-tool aesthetic), deliberately *not* the Web3
neon-gradient look, gold as the single accent color, Fraunces for display
type, Geist for body text, IBM Plex Mono for anything chain-related.

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


