# TODOS

## CLI/SDK distribution pipeline
**What:** Publish the CLI and SDK to real distribution channels (npm for SDK,
crates.io or GitHub Releases for CLI binaries).
**Why:** Breadth-scope CLI/SDK (build sequence step 8) are worthless if
nobody can actually install them — "no placeholder code" applies to
distribution too, not just implementation.
**Pros:** Completes the real-deliverable bar for breadth items; makes the
CLI/SDK usable by anyone, not just people who clone the repo and build from
source.
**Cons:** Real CI/publishing setup work (versioning, release automation,
cross-platform builds); not core-path blocking.
**Context:** Flagged during `/plan-eng-review` (2026-07-04) Step 0 distribution
check — the design doc's Distribution Plan covered the 11 backend services,
Solana program, and frontend, but never addressed how CLI/SDK reach users.
**Depends on:** gateway's API surface being stable (build sequence step 7+).
**Status:** Deferred — not core-path blocking, added here so it isn't
silently forgotten once breadth work starts.

## Squads multisig test fails in Docker-on-Windows local validator
**What:** `tests/multisig_authority.ts`'s Squads 2-of-3 multisig test times out
in its `before all` hook (multisig creation transaction never confirms).
**Why:** The local `solana-test-validator`'s own vote/consensus mechanism
repeatedly fails its threshold check (`Couldn't vote on heaviest fork`,
1,300+ occurrences in one run) under Docker Desktop on Windows/WSL2. Anchor's
own `.rpc()` calls (used by the 6 passing single-authority tests) tolerate
this; the Squads SDK's `confirmTransaction` calls do not. Confirmed not a
program or test-code bug: the `agent_trust` program itself is fully proven
correct (registration, score bounds, authority checks, DID length limits,
audit hash recording all pass against this same validator).
**Pros of fixing now:** would give full test coverage of the actual 2-of-3
security model this whole step 2 build phase was meant to prove.
**Cons:** looks like validator/infra timing under this specific
containerized-Windows setup, not something fixable in the program or test
code — needs either a non-Windows-Docker environment or deeper
`solana-test-validator` flag tuning to actually resolve.
**Context:** Hit during blockchain-service + Solana program build (build
sequence step 2), 2026-07-04. User decision: accept as known limitation,
move on to the next build step.
**Status:** Deferred. Retry on native Linux/WSL2 (not Docker Desktop) or a
cloud CI runner before considering the multisig integration itself unproven.

## Light mode toggle
**What:** Add a light theme alongside the dark theme, with a user-facing
toggle.
**Why:** The demo's aesthetic direction is explicitly dark-only (matches
OpenAI/Stripe/Linear/Vercel/Anthropic inspiration), but enterprise buyers
often expect light mode as an option, and the "Enterprise Ready" badge
feature implies enterprise-facing use down the line.
**Pros:** Broader accessibility/preference coverage; more "enterprise
credible" for organizations with light-mode-only style guides.
**Cons:** Real design + implementation cost (a full second color system,
every component re-tested against both themes); dilutes the deliberate dark,
glassmorphic brand identity established for the demo.
**Context:** Flagged during `/plan-design-review` (2026-07-04) Pass 6 —
deferred rather than silently decided either way.
**Depends on:** DESIGN.md existing first (via `/design-consultation`) so a
light theme is derived from the same token system, not designed separately.
**Status:** Deferred — dark-only for v1.
