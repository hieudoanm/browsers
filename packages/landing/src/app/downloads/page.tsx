import type { Metadata } from 'next';
import type { FC, ReactNode } from 'react';
import type { IconType } from 'react-icons';
import { FaApple, FaDownload, FaLinux, FaTerminal } from 'react-icons/fa';

export const metadata: Metadata = {
  title: 'Downloads',
  description:
    'Download browserverless — the Servo-powered browser runtime CLI — for macOS and Linux.',
  alternates: { canonical: '/downloads/' },
};

const GITHUB = 'https://github.com/hieudoanm/browsers';
const RELEASES = `${GITHUB}/releases/latest/download`;
const RELEASE_PAGE = `${GITHUB}/releases/latest`;

interface Asset {
  label: string;
  href: string;
}

interface Cell {
  name: string;
  icon: IconType;
  desc: string;
  assets: Asset[];
}

const CLI: Cell[] = [
  {
    name: 'macOS (Silicon)',
    icon: FaApple,
    desc: 'Apple Silicon Macs — M1 and later.',
    assets: [
      {
        label: 'Apple Silicon',
        href: `${RELEASES}/cli-browsers-aarch64-apple-darwin`,
      },
    ],
  },
  {
    name: 'macOS (Intel)',
    icon: FaApple,
    desc: 'Intel-based Macs.',
    assets: [
      { label: 'Intel', href: `${RELEASES}/cli-browsers-x86_64-apple-darwin` },
    ],
  },
  {
    name: 'Linux',
    icon: FaLinux,
    desc: 'x86_64 GNU/Linux distributions.',
    assets: [
      {
        label: 'x86_64',
        href: `${RELEASES}/cli-browsers-x86_64-unknown-linux-gnu`,
      },
    ],
  },
];

function Section({
  icon: Icon,
  title,
  children,
}: {
  icon: IconType;
  title: string;
  children: ReactNode;
}) {
  return (
    <div className="mb-12 last:mb-0">
      <h2 className="text-base-content mb-6 flex items-center gap-2 text-2xl leading-tight">
        <Icon className="text-primary h-6 w-6" />
        {title}
      </h2>
      {children}
    </div>
  );
}

function CardGrid({ cells, cols }: { cells: Cell[]; cols: string }) {
  const visible = cells.filter((c) => c.assets.length > 0);
  if (visible.length === 0) return null;
  return (
    <div className={`grid grid-cols-1 gap-6 sm:grid-cols-2 ${cols}`}>
      {visible.map((c) => (
        <div
          key={c.name}
          className="card border-base-300 bg-base-100 border p-6">
          <div className="bg-primary/10 text-primary mb-3.5 flex h-11 w-11 items-center justify-center rounded-lg">
            <c.icon className="h-6 w-6" />
          </div>
          <h3 className="text-base-content m-0 text-lg leading-tight">
            {c.name}
          </h3>
          <p className="text-base-content/60 mt-1 line-clamp-2 text-sm">
            {c.desc}
          </p>
          <div className="mt-4 flex flex-col gap-2">
            {c.assets.map((a) => (
              <a
                key={a.href}
                href={a.href}
                className="btn btn-primary btn-sm gap-2">
                <FaDownload className="h-3.5 w-3.5" />
                {a.label}
              </a>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

const Downloads: FC = () => (
  <main>
    <section className="py-20">
      <div className="mx-auto w-full max-w-6xl px-5">
        <div className="mx-auto mb-12 max-w-2xl text-center">
          <h1 className="m-0 text-4xl leading-tight tracking-normal text-balance">
            Download browserverless
          </h1>
          <p className="mt-3 text-lg text-gray-500">
            Prebuilt binaries for headless screenshots, scraping, and MCP tools.
            Assets are in the{' '}
            <a href={RELEASE_PAGE} className="text-primary">
              latest release
            </a>
            .
          </p>
        </div>

        <Section icon={FaTerminal} title="CLI">
          <CardGrid cells={CLI} cols="lg:grid-cols-4" />
        </Section>

        <p className="text-base-content/60 mt-8 text-center text-sm">
          Or build from source:{' '}
          <code className="bg-base-300 rounded px-1.5 py-0.5 font-mono text-xs">
            cargo build --release -p browsers-cli
          </code>{' '}
          in the{' '}
          <a href={GITHUB} className="text-primary">
            repository
          </a>
          .
        </p>
      </div>
    </section>
  </main>
);

export default Downloads;
