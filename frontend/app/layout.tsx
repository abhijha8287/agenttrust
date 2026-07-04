import type { Metadata } from 'next';
import { Fraunces, Geist, IBM_Plex_Mono } from 'next/font/google';
import './globals.css';

const fraunces = Fraunces({
  subsets: ['latin'],
  variable: '--font-display',
  weight: ['400', '500', '600', '700'],
});

const geist = Geist({
  subsets: ['latin'],
  variable: '--font-body',
  weight: ['400', '500', '600', '700'],
});

const plexMono = IBM_Plex_Mono({
  subsets: ['latin'],
  variable: '--font-mono',
  weight: ['400', '500', '600'],
});

export const metadata: Metadata = {
  title: 'AgentTrust — The trust layer for autonomous AI agents',
  description:
    'Cryptographic identity, reputation scoring, and on-chain audit trails for autonomous AI agents.',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body
        className={`${fraunces.variable} ${geist.variable} ${plexMono.variable}`}
        style={{ fontFamily: 'var(--font-body)' }}
      >
        {children}
      </body>
    </html>
  );
}
