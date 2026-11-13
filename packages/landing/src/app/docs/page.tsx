import type { Metadata, NextPage } from 'next';
import type { FC } from 'react';
import Markdown from '@/components/shared/Markdown';
import { getDoc } from '@/lib/docs';

export const metadata: Metadata = {
  title: 'Documentation',
};

const DocsIndexPage: FC & NextPage = () => {
  const doc = getDoc('index');
  if (!doc) return null;
  return <Markdown content={doc.content} />;
};

export default DocsIndexPage;
