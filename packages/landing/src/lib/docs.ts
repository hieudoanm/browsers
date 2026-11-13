import fs from 'node:fs';
import path from 'node:path';

export type Doc = {
  slug: string;
  title: string;
  description?: string;
  order?: number;
  group?: string;
  content: string;
};

const docsDirectory = path.join(process.cwd(), 'md', 'docs');
const slugSegmentPattern = /^[a-z0-9-]+$/i;
const docFilePattern = /\.md$/i;

const parseFrontmatter = (
  raw: string,
): {
  data: Record<string, string>;
  body: string;
} => {
  if (!raw.startsWith('---')) {
    return { data: {}, body: raw };
  }
  const parts = raw.split('---');
  if (parts.length < 3) {
    return { data: {}, body: raw };
  }
  const frontmatter = parts[1];
  const body = parts.slice(2).join('---').replace(/^\n+/, '');
  const data: Record<string, string> = {};
  for (const line of frontmatter.split('\n')) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#')) continue;
    const idx = trimmed.indexOf(':');
    if (idx === -1) continue;
    const key = trimmed.slice(0, idx).trim();
    let value = trimmed.slice(idx + 1).trim();
    value = value.replace(/^['"]|['"]$/g, '');
    data[key] = value;
  }
  return { data, body };
};

const walkDocFiles = (dir: string, prefix: string): string[] => {
  const files: string[] = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (entry.name.startsWith('.')) continue;
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...walkDocFiles(fullPath, `${prefix}${entry.name}/`));
    } else if (
      entry.isFile() &&
      docFilePattern.test(entry.name) &&
      entry.name.toLowerCase() !== 'index.md'
    ) {
      files.push(`${prefix}${entry.name}`);
    }
  }
  return files;
};

const readDoc = (relativePath: string): Doc => {
  const slug = relativePath.replace(docFilePattern, '');
  const raw = fs.readFileSync(path.join(docsDirectory, relativePath), 'utf8');
  const { data, body } = parseFrontmatter(raw);
  const heading = body.match(/^#\s+(.+)$/m)?.[1]?.trim();
  const title =
    data.title?.trim() ||
    heading ||
    slug
      .split(/[/-]/)
      .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
      .join(' ');

  const order = data.order !== undefined ? Number(data.order) : undefined;
  const group = slug.includes('/') ? slug.split('/')[0] : undefined;

  return {
    slug,
    title,
    description: data.description,
    order: Number.isNaN(order as number) ? undefined : order,
    group,
    content: body,
  };
};

export const getDocs = (): Doc[] => {
  return walkDocFiles(docsDirectory, '')
    .map(readDoc)
    .sort((left, right) => {
      if (left.order !== undefined && right.order !== undefined) {
        return left.order - right.order;
      }
      if (left.order !== undefined) return -1;
      if (right.order !== undefined) return 1;
      return left.title.localeCompare(right.title, undefined, { numeric: true });
    });
};

export const getDoc = (slug: string): Doc | undefined => {
  const segments = slug.split('/');
  const valid =
    segments.every((segment) => slugSegmentPattern.test(segment)) &&
    (slug === 'index' ||
      segments.every((segment) => segment.toLowerCase() !== 'index'));
  if (!valid) return undefined;
  const relativePath = `${slug}.md`;
  const filePath = path.join(docsDirectory, relativePath);
  if (!filePath.startsWith(docsDirectory) || !fs.existsSync(filePath)) {
    return undefined;
  }
  return readDoc(relativePath);
};
