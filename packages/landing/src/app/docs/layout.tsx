import type { Metadata } from 'next';
import Link from 'next/link';
import type { FC, ReactNode } from 'react';
import { getDocs } from '@/lib/docs';

export const metadata: Metadata = {
  title: 'Documentation',
  description:
    'Guides for installing browserverless, using the CLI, the HTTP API, and the MCP server.',
};

interface DocsLayoutProps {
  children: ReactNode;
}

const DocsLayout: FC<DocsLayoutProps> = ({ children }) => {
  const docs = getDocs();

  return (
    <div className="mx-auto flex w-full max-w-6xl gap-10 px-5 py-14">
      <aside className="hidden w-56 shrink-0 md:block">
        <nav className="sticky top-24 space-y-1">
          <p className="text-base-content/50 px-3 pb-2 text-xs font-bold tracking-widest uppercase">
            Documentation
          </p>
          <Link
            href="/docs"
            className="text-base-content/70 hover:bg-base-200 hover:text-primary block rounded-lg px-3 py-2 text-sm font-semibold">
            Overview
          </Link>
          {docs.map((doc) => (
            <Link
              key={doc.slug}
              href={`/docs/${doc.slug}`}
              className="text-base-content/70 hover:bg-base-200 hover:text-primary block rounded-lg px-3 py-2 text-sm font-semibold">
              {doc.title}
            </Link>
          ))}
        </nav>
      </aside>
      <article className="bg-base-100 border-base-300 min-w-0 flex-1 rounded-2xl border p-8 md:p-12">
        {children}
      </article>
    </div>
  );
};

export default DocsLayout;
