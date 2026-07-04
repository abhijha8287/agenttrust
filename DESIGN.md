# Design System — AgentTrust

## Product Context
- **What this is:** The decentralized trust layer for autonomous AI agents —
  cryptographic identity, reputation scoring, and on-chain audit trails.
- **Who it's for:** Hackathon judges evaluating the demo first; developers
  and enterprises adopting it after.
- **Space/industry:** AI agent infrastructure / Web3 identity & trust.
- **Project type:** Hybrid — marketing landing page + data-dense app
  (marketplace, agent profile, dashboard).
- **The memorable thing:** "This is real, verifiable trust — not vibes."
  Every design decision below serves this. If a choice adds decoration
  without adding credibility, cut it.

## Aesthetic Direction
- **Direction:** Technical Restraint — Linear/Stripe/Vercel-inspired dark
  dev-tool aesthetic. **Explicitly not** the Web3 cyber-gradient/neon-glow
  convention.
- **Decoration level:** Intentional — a subtle blueprint-grid background
  texture signals engineering seriousness; no aurora/glow effects, no
  particle canvases.
- **Mood:** Audited, credible, quietly confident. A trust certificate, not
  a hype poster.
- **Why not Web3-neon:** [EUREKA, logged 2026-07-04] Every crypto-native
  product defaults to neon cyber-gradients (acid green, neon violet, laser
  blue) because that's the visual shorthand for "crypto-native" to a crypto
  audience. AgentTrust's entire pitch is the opposite of hype — dressing a
  verifiability product in hype-coded visuals undercuts the message before
  anyone reads a word. Blockchain is treated as invisible, credible
  infrastructure (one "Verify on Solscan" link), never the dominant visual
  theme.
- **Reference sites:** Linear, Stripe, Vercel (dark theme, blueprint-grid
  aesthetic, product-first hero, restrained accent usage).

## Typography
- **Display/Hero:** Fraunces — serif, variable weight/optical size. Chosen
  for certificate/audit-document gravitas ("reads like a well-designed tax
  report," not another SaaS geometric-sans hero). A deliberate risk: most
  competitors in this space use all-geometric-sans type.
- **Body/UI:** Geist — clean, readable, technical without being cold.
- **Data/Tables/Addresses:** IBM Plex Mono — wallet addresses, tx
  signatures, audit hashes, trust scores. Functional monospace (tabular
  alignment), not decorative.
- **Code:** IBM Plex Mono (shared with data — one less typeface to load).
- **Loading:** Google Fonts CDN
  (`family=Fraunces:opsz,wght@9..144,400;9..144,500;9..144,600;9..144,700&family=Geist:wght@400;500;600;700&family=IBM+Plex+Mono:wght@400;500;600`)
- **Scale:** Hero 56px / H2 28-32px / Body 16px / Small 13-14px / Mono data
  13-14px. Fluid scaling via `clamp()` recommended for hero at implementation.
- **Blacklisted:** Never use Inter, Roboto, Arial, system-ui/-apple-system
  as a PRIMARY font (fallback stack only). Never Papyrus/Comic
  Sans/decorative script fonts.

## Color
- **Approach:** Restrained — one brand accent, semantic colors reserved
  strictly for trust-tier signaling (never reused as decoration).
- **Background:** `#0A0A0B` (near-black base)
- **Surface:** `#17171B` (cards, panels)
- **Surface raised:** `#1D1D22` (avatars, nested elements)
- **Border:** `#29292F` (default) / `#3A3A42` (strong, inputs/hover)
- **Text primary:** `#F2F1EC` (warm off-white, pairs with gold accent)
- **Text muted:** `#93939C`
- **Text faint:** `#5C5C64`
- **Accent (Gold):** `#D9A441` — brand accent, CTAs, links, Verified
  badges. Deliberately ties to the product's own "Gold Badge" reputation
  tier — trust is earned, like a seal. Used sparingly; never as a gradient
  or glow.
- **Accent bright (hover):** `#F0C868`
- **Semantic — Success/High Trust:** `#22C55E` (bg dim `#16311F`)
- **Semantic — Warning/Low Confidence:** `#F59E0B` (bg dim `#3A2A0D`)
- **Semantic — Danger/At Risk:** `#EF4444` (bg dim `#3A1717`)
- **Dark mode:** This is a dark-only product for v1 (light mode deferred —
  see TODOS.md). Do not build a light theme without revisiting this file
  first — a real light palette needs to be derived from these tokens, not
  invented separately.

## Spacing
- **Base unit:** 8px
- **Density:** Comfortable for marketing/profile pages, slightly more
  compact for dashboard/marketplace data density.
- **Scale:** 2xs(4) sm(8) md(16) lg(24) xl(32) 2xl(48) 3xl(64) 4xl(96)

## Layout
- **Approach:** Grid-disciplined for app screens (marketplace, profile,
  dashboard); slightly looser/editorial for the landing hero.
- **Grid:** 12-column at desktop, collapses per component at tablet/mobile
  breakpoints (see `/plan-design-review`'s responsive pass for per-screen
  detail — Agent Profile's sticky nav collapses to a horizontal pill nav
  below 768px).
- **Max content width:** 1100px
- **Border radius:** sm(4px) components, md(6px) inputs/tags, lg(10px)
  cards. Deliberately restrained, not bubbly — reinforces the "serious
  infrastructure" posture and avoids the AI-slop uniform-large-radius
  pattern.
- **Background texture:** Subtle 48px grid pattern (1px lines, `--border`
  color) as the page background — the "blueprint" aesthetic signal.

## Motion
- **Approach:** Intentional — motion earns its place, never decorative for
  its own sake.
- **Signature moment:** The trust gauge animates its fill on load (0 → 
  current score) and on live WebSocket update (old value → new value,
  ~400ms ease, brief highlight flash). This is the demo's visual "whoa"
  moment — never skip the animation even when data arrives instantly.
- **Marketplace cards:** Staggered fade/slide-in on scroll, not all at once.
- **Dashboard feed:** New executions slide down from the top, existing rows
  shift — never a jarring re-render.
- **Easing:** enter(ease-out) exit(ease-in) move(ease-in-out)
- **Duration:** micro(50-100ms) short(150-250ms) medium(250-400ms
  — gauge fill) long(400-700ms)

## Signature Component — Trust Gauge
Radial SVG ring (0-100), color-coded by tier (danger/warning/success),
numeric score centered, tier label below. Two sizes: 44px (marketplace card
mini-indicator) and 140px (agent profile hero). One component, reused
everywhere trust is shown — this is the product's single most important
piece of visual identity. Must carry an ARIA label announcing the actual
score and tier ("Trust score: 87 out of 100, Gold tier") — a purely visual
gauge with no text equivalent fails accessibility for this component.

## Decisions Log
| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-07-04 | Technical Restraint over Web3-neon aesthetic | EUREKA insight — hype-coded visuals undercut a verifiability-first pitch |
| 2026-07-04 | Gold/amber single accent, not blue/purple | Ties to product's own Gold Badge tier; avoids blacklisted purple, avoids generic SaaS blue |
| 2026-07-04 | Fraunces serif display, deliberate risk | Certificate/audit-document gravitas; differentiates from all-sans convention |
| 2026-07-04 | Radial gauge as signature component | Animates the live-update "whoa" moment; reused at 2 sizes for consistency |
| 2026-07-04 | Dark-only for v1 | Matches committed aesthetic direction; light mode deferred to TODOS.md |
