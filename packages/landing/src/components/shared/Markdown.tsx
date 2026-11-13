'use client';

import type { FC } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

const Markdown: FC<{ content: string }> = ({ content }) => {
  return (
    <div
      data-testid="markdown"
      className="prose prose-slate max-w-none prose-headings:scroll-mt-24 prose-a:text-primary prose-code:rounded prose-code:bg-base-300 prose-code:px-1.5 prose-code:py-0.5 prose-code:font-mono prose-code:text-sm prose-code:before:content-none prose-code:after:content-none prose-pre:bg-neutral prose-pre:text-neutral-content">
      <ReactMarkdown remarkPlugins={[remarkGfm]}>{content}</ReactMarkdown>
    </div>
  );
};

export default Markdown;
