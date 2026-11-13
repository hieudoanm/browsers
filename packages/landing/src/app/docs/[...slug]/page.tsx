import type { Metadata } from 'next';
import Link from 'next/link';
import { notFound } from 'next/navigation';
import type { FC } from 'react';
import Markdown from '@/components/shared/Markdown';
import { getDoc, getDocs } from '@/lib/docs';

export const dynamicParams = false;

export const generateStaticParams = () => {
  return getDocs().map((doc) => ({ slug: doc.slug.split('/') }));
};

export const generateMetadata = async ({
  params,
}: {
  params: Promise<{ slug: string[] }>;
}): Promise<Metadata> => {
  const { slug } = await params;
  const doc = getDoc(slug.join('/'));
  return doc ? { title: doc.title } : {};
};

interface DocPageProps {
  params: Promise<{ slug: string[] }>;
}

const DocPage: FC<DocPageProps> = async ({ params }) => {
  const { slug: segments } = await params;
  const slug = segments.join('/');
  const doc = getDoc(slug);
  if (!doc) notFound();

  const docs = getDocs();
  const currentIndex = docs.findIndex((d) => d.slug === slug);
  const previous = currentIndex > 0 ? docs[currentIndex - 1] : undefined;
  const next =
    currentIndex >= 0 && currentIndex < docs.length - 1
      ? docs[currentIndex + 1]
      : undefined;

  return (
    <div>
      <Markdown content={doc.content} />
      <nav className="border-base-300 mt-12 flex items-center justify-between gap-4 border-t pt-6">
        <div className="flex-1">
          {previous ? (
            <Link
              href={`/docs/${previous.slug}`}
              className="border-base-300 group flex items-center gap-2 rounded-lg border px-4 py-2 text-sm">
              <span aria-hidden="true">←</span>
              <span className="flex flex-col">
                <span className="text-base-content/50 text-xs">Previous</span>
                <span className="font-medium">{previous.title}</span>
              </span>
            </Link>
          ) : null}
        </div>
        <div className="flex-1 text-right">
          {next ? (
            <Link
              href={`/docs/${next.slug}`}
              className="border-base-300 group ml-auto flex items-center justify-end gap-2 rounded-lg border px-4 py-2 text-sm">
              <span className="flex flex-col items-end">
                <span className="text-base-content/50 text-xs">Next</span>
                <span className="font-medium">{next.title}</span>
              </span>
              <span aria-hidden="true">→</span>
            </Link>
          ) : null}
        </div>
      </nav>
    </div>
  );
};

export default DocPage;
