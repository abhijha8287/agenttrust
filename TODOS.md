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
