'use client';

import { useEffect, useRef } from 'react';

// Tier bands aren't exposed by the API (trust_score is just an integer
// 0-10000 on-chain / 0-100 here) — this mapping is a client-side display
// convention only, consistent with DESIGN.md's semantic color reservations
// (success = high trust, warning = low confidence, danger = at risk).
function tierFor(score: number): { label: string; color: string } {
  if (score >= 70) return { label: 'Gold Tier', color: 'var(--success)' };
  if (score >= 40) return { label: 'Silver Tier', color: 'var(--warning)' };
  return { label: 'Unranked', color: 'var(--danger)' };
}

export function TrustGauge({
  score,
  size = 140,
}: {
  score: number;
  size?: number;
}) {
  const fillRef = useRef<SVGCircleElement>(null);
  const clamped = Math.max(0, Math.min(100, score));
  const strokeWidth = (size * 8) / 140;
  const radius = size / 2 - strokeWidth;
  const circumference = 2 * Math.PI * radius;
  const targetOffset = circumference * (1 - clamped / 100);
  const { label, color } = tierFor(clamped);
  const isHero = size >= 100;

  useEffect(() => {
    const el = fillRef.current;
    if (!el) return;
    // Animate 0 -> score on mount, per DESIGN.md's signature "whoa" moment —
    // set full offset first, then transition to target on next frame.
    el.style.transition = 'none';
    el.style.strokeDashoffset = String(circumference);
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        el.style.transition = 'stroke-dashoffset 1.1s cubic-bezier(0.16, 1, 0.3, 1)';
        el.style.strokeDashoffset = String(targetOffset);
      });
    });
  }, [circumference, targetOffset]);

  return (
    <div
      role="img"
      aria-label={`Trust score: ${clamped} out of 100, ${label}`}
      style={{ position: 'relative', width: size, height: size, flexShrink: 0 }}
    >
      <svg
        width={size}
        height={size}
        viewBox={`0 0 ${size} ${size}`}
        aria-hidden="true"
        style={{ transform: 'rotate(-90deg)' }}
      >
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="none"
          stroke="var(--border)"
          strokeWidth={strokeWidth}
        />
        <circle
          ref={fillRef}
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="none"
          stroke={color}
          strokeWidth={strokeWidth}
          strokeLinecap="round"
          strokeDasharray={circumference}
          strokeDashoffset={circumference}
        />
      </svg>
      <div
        style={{
          position: 'absolute',
          inset: 0,
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
        }}
      >
        <div
          style={{
            fontFamily: 'var(--font-mono)',
            fontWeight: 600,
            fontSize: isHero ? 34 : Math.max(10, size * 0.28),
            color,
          }}
        >
          {clamped}
        </div>
        {isHero && (
          <div
            style={{
              fontSize: 10,
              color: 'var(--text-muted)',
              textTransform: 'uppercase',
              letterSpacing: '0.06em',
              marginTop: 2,
            }}
          >
            {label}
          </div>
        )}
      </div>
    </div>
  );
}
