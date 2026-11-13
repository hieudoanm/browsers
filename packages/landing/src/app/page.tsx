import type { Metadata, NextPage } from 'next';
import type { FC } from 'react';
import CTA from '@/components/molecules/CTA';
import Features, { type Feature } from '@/components/molecules/Features';
import Hero from '@/components/molecules/Hero';
import SectionHead from '@/components/molecules/SectionHead';
import Terminal from '@/components/molecules/Terminal';
import {
  LuCamera,
  LuFileText,
  LuGlobe,
  LuLayoutGrid,
  LuPlug,
  LuSquareTerminal,
} from 'react-icons/lu';

export const metadata: Metadata = {
  alternates: { canonical: '/' },
};

const GITHUB = 'https://github.com/hieudoanm/browsers';

const FEATURES: Feature[] = [
  {
    icon: LuGlobe,
    title: 'Real browser engine',
    body: 'Built on Servo — the same parallel browser engine Firefox is based on. Full HTML, CSS, and JavaScript support with no Chromium or WebKit dependency.',
  },
  {
    icon: LuCamera,
    title: 'Headless screenshots',
    body: 'Render any URL to a PNG entirely offscreen. No window, no desktop session, no monitor — safe for CI and servers.',
  },
  {
    icon: LuFileText,
    title: 'Scrape to clean HTML',
    body: 'Extract the fully rendered DOM of a page after JavaScript execution, with final URL, title, and load status attached.',
  },
  {
    icon: LuSquareTerminal,
    title: 'One static binary',
    body: 'A single Rust binary covers the CLI, the HTTP server, and the MCP server. Cross-compiled releases for Linux and macOS.',
  },
  {
    icon: LuPlug,
    title: 'HTTP API + MCP',
    body: 'Serve /api/v1/screenshot and /api/v1/scrape over HTTP, or plug the same engine into AI agents over Model Context Protocol on stdio.',
  },
  {
    icon: LuLayoutGrid,
    title: 'Headed mode too',
    body: 'The same codebase opens an interactive desktop window via winit when you need to see the page, not just capture it.',
  },
];

const HERO_LINES = [
  '$ browserverless screenshot https://example.com -o example.png',
  'Screenshot saved to example.png',
  '$ browserverless serve --bind 127.0.0.1:8080',
  'listening on http://127.0.0.1:8080',
];

const DEMO_TERMINALS: { caption: string; lines: string[]; wide?: boolean }[] = [
  {
    caption: 'Screenshot a page',
    lines: [
      '$ browserverless screenshot https://example.com \\',
      '    --output example.png --width 1280 --height 720',
      'Screenshot saved to example.png',
    ],
  },
  {
    caption: 'Scrape rendered HTML',
    lines: [
      '$ browserverless scrape https://example.com -o page.html',
      'Wrote 84,213 bytes to page.html',
    ],
  },
  {
    caption: 'Serve the HTTP API',
    lines: [
      '$ browserverless serve --bind 127.0.0.1:8080',
      '[2026-11-13 09:12:04] req=1 POST /api/v1/scrape -> 200 (459ms)',
      '[2026-11-13 09:12:11] req=2 POST /api/v1/screenshot -> 200 (1.2s)',
    ],
  },
  {
    caption: 'MCP for AI agents',
    wide: true,
    lines: [
      '$ browserverless mcp serve',
      '{"jsonrpc":"2.0","id":1,"method":"tools/call",',
      ' "params":{"name":"browserverless_screenshot",',
      ' "arguments":{"url":"https://example.com"}}}',
      '{"jsonrpc":"2.0","id":1,"result":{"content":[',
      ' {"type":"image","data":"iVBORw0KGgoAAAANSUhEUg..."}]}}',
    ],
  },
];

const STEPS = [
  {
    title: 'Pick a mode',
    body: 'Use the CLI for one-off captures, serve for an HTTP API, mcp serve for AI agents, or open for an interactive window. Same engine underneath.',
    snippet: 'browserverless screenshot|scrape|serve|mcp|open',
  },
  {
    title: 'Point it at a URL',
    body: 'Every command takes an absolute http(s) URL — or a file:// URL for local fixtures. Servo loads the page, runs its scripts, and waits for layout.',
    snippet: 'browserverless screenshot https://example.com',
  },
  {
    title: 'Get pixels or markup',
    body: 'Screenshots come back as PNG at your chosen viewport; scrapes return the fully rendered DOM as HTML. The HTTP API wraps both with metadata headers.',
    snippet: 'POST /api/v1/screenshot  {"url": "https://..."}',
  },
];

const INSTALL_LINES = [
  '$ gh release download latest \\',
  '    --repo hieudoanm/browsers \\',
  '    --pattern "browserverless-*"',
  '$ chmod +x browserverless-x86_64-unknown-linux-gnu',
  '$ ./browserverless-x86_64-unknown-linux-gnu screenshot https://example.com -o page.png',
  'Screenshot saved to page.png',
];

const StepCard: FC<{ step: (typeof STEPS)[number]; index: number }> = ({
  step,
  index,
}) => (
  <article className="card border-base-300 bg-base-100 rounded-2xl border p-6">
    <span className="text-accent mb-2 block font-mono text-sm font-semibold">
      {String(index + 1).padStart(2, '0')}
    </span>
    <h3 className="text-base-content m-0 mb-2 text-lg font-bold">
      {step.title}
    </h3>
    <p className="text-base-content/60 text-sm">{step.body}</p>
    <code className="border-base-300 bg-base-200 text-base-content/60 mt-4 block overflow-x-auto rounded-lg border px-3 py-2 font-mono text-xs">
      {step.snippet}
    </code>
  </article>
);

const HomePage: NextPage = () => (
  <main>
    <Hero
      title="A browser, without the window."
      description="browserverless embeds the Servo engine in a single Rust binary — screenshot pages, scrape rendered HTML, serve an HTTP API, or drive it from AI agents over MCP. Headed mode included when you need to look."
      primary={{ label: 'Get started', href: '#install' }}
      secondary={{ label: 'Read the docs', href: '/docs' }}
      stats={[
        { value: 'Servo', label: 'engine, not a wrapper' },
        { value: '1', label: 'static binary' },
        { value: 'HTTP + MCP', label: 'two APIs, one engine' },
      ]}>
      <Terminal lines={HERO_LINES} />
    </Hero>

    <Features
      title="Features"
      description="A thin, honest embedder around a real browser engine — no Chromium fork, no Electron, no cloud calls."
      features={FEATURES}
    />

    <section id="demo" className="bg-base-200 py-20">
      <div className="mx-auto w-full max-w-6xl px-5">
        <SectionHead
          title="See it in action"
          description="One binary, four ways to drive the same engine."
        />
        <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
          {DEMO_TERMINALS.map((t) => (
            <Terminal
              key={t.caption}
              lines={t.lines}
              className={t.wide ? 'lg:col-span-2' : ''}
            />
          ))}
        </div>
      </div>
    </section>

    <section id="how-it-works" className="py-20">
      <div className="mx-auto w-full max-w-6xl px-5">
        <SectionHead
          title="How it works"
          description="URL in, pixels or markup out — Servo does the rendering."
        />
        <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {STEPS.map((step, i) => (
            <StepCard key={step.title} step={step} index={i} />
          ))}
        </div>
      </div>
    </section>

    <section id="install" className="bg-base-200 py-20">
      <div className="mx-auto w-full max-w-3xl px-5">
        <SectionHead
          title="Get started"
          description="Grab a prebuilt binary from GitHub Releases, or build from source with Cargo. Requires a C toolchain for Servo's system dependencies on Linux."
        />
        <Terminal lines={INSTALL_LINES} />
        <p className="text-base-content/60 mt-6 text-center text-sm">
          Or build from source:{' '}
          <code className="bg-base-300 rounded px-1.5 py-0.5 font-mono text-xs">
            cargo build --release -p browsers-cli
          </code>{' '}
          — see the{' '}
          <a
            href={`${GITHUB}/blob/master/README.md`}
            target="_blank"
            rel="noreferrer"
            className="text-primary font-semibold">
            README
          </a>{' '}
          for details.
        </p>
      </div>
    </section>

    <CTA
      title="Ready to render?"
      description="One command turns any URL into a screenshot, markup, or an MCP tool call."
      primary={{ label: 'View on GitHub', href: GITHUB, external: true }}
      secondary={{
        label: 'Browse the docs',
        href: '/docs',
      }}
    />
  </main>
);

export default HomePage;
