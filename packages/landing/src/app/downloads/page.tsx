import type { Metadata } from 'next';
import type { FC } from 'react';
import type { IconType } from 'react-icons';
import { FaApple, FaDownload, FaLinux } from 'react-icons/fa';

export const metadata: Metadata = {
  title: 'Downloads',
  description:
    'Download browserverless — the Servo-powered browser runtime CLI — for macOS and Linux.',
  alternates: { canonical: '/downloads/' },
};

const GITHUB = 'https://github.com/hieudoanm/browsers';
const RELEASES = `${GITHUB}/releases/latest/download`;
const RELEASE_PAGE = `${GITHUB}/releases/latest`;

interface Platform {
  name: string;
  icon: IconType;
  assets: { label: string; href: string }[];
}

const PLATFORMS: Platform[] = [
  {
    name: 'macOS',
    icon: FaApple,
    assets: [
      {
        label: 'Apple Silicon',
        href: `${RELEASES}/cli-browsers-aarch64-apple-darwin`,
      },
    ],
  },
  {
    name: 'Linux',
    icon: FaLinux,
    assets: [
      {
        label: 'x86_64',
        href: `${RELEASES}/cli-browsers-x86_64-unknown-linux-gnu`,
      },
    ],
  },
];

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
            . Windows builds are not published — Servo&apos;s GTK dependencies
            make them impractical.
          </p>
        </div>

        <div className="grid grid-cols-1 gap-6 sm:grid-cols-2">
          {PLATFORMS.map((p) => (
            <div
              key={p.name}
              className="card border-base-300 bg-base-100 border p-6">
              <div className="bg-primary/10 text-primary mb-3.5 flex h-11 w-11 items-center justify-center rounded-lg">
                <p.icon className="h-6 w-6" />
              </div>
              <h2 className="text-base-content m-0 mb-1 text-lg leading-tight">
                {p.name}
              </h2>
              <div className="mt-4 flex flex-col gap-2">
                {p.assets.map((a) => (
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
