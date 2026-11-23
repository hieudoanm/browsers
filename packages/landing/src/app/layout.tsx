import type { Metadata } from 'next';
import { Inter } from 'next/font/google';
import type { FC, ReactNode } from 'react';
import Footer from '@/components/molecules/Footer';
import Navbar from '@/components/molecules/Navbar';
import { getDocPages } from '@/lib/docs';
import './globals.css';

const inter = Inter({ subsets: ['latin'] });

const GITHUB = 'https://github.com/hieudoanm/browsers';
const BASE = process.env.NODE_ENV === 'production' ? '/browsers' : '';
const LOGO = `${BASE}/logo.svg`;

const NAV_LINKS = [
  { label: 'Features', href: '#features' },
  { label: 'Demo', href: '#demo' },
  { label: 'Install', href: '#install' },
  { label: 'Downloads', href: '/downloads/' },
];

const DESCRIPTION =
  'A Servo-powered browser runtime as a library, a CLI, and an HTTP server — headless screenshots, scraping, and MCP tools from one Rust binary.';

export const metadata: Metadata = {
  metadataBase: new URL('https://hieudoanm.github.io/browsers/'),
  title: {
    default: 'browserverless — a browser without the window',
    template: '%s — browserverless',
  },
  description: DESCRIPTION,
  keywords: [
    'headless browser',
    'browser automation',
    'serverless browser',
    'Rust',
    'web scraping',
  ],
  authors: [{ name: 'hieudoan', url: 'https://github.com/hieudoanm/browsers' }],
  category: 'software',
  openGraph: {
    siteName: 'browserverless',
    type: 'website',
    title: 'browserverless — a browser without the window',
    description: DESCRIPTION,
    url: 'https://hieudoanm.github.io/browsers/',
    images: ['/og.png'],
  },
  twitter: { card: 'summary_large_image', images: ['/og.png'] },
  icons: {
    icon: [
      { url: `${BASE}/favicon.ico`, sizes: 'any' },
      { url: `${BASE}/favicon.svg`, type: 'image/svg+xml' },
    ],
  },
};

const JSON_LD = {
  '@context': 'https://schema.org',
  '@graph': [
    {
      '@type': 'WebSite',
      name: 'browserverless',
      url: 'https://hieudoanm.github.io/browsers/',
      description:
        'A Servo-powered browser runtime as a library, a CLI, and an HTTP server — headless screenshots, scraping, and MCP tools from one Rust binary.',
    },
    {
      '@type': 'SoftwareApplication',
      name: 'browserverless',
      applicationCategory: 'UtilitiesApplication',
      description:
        'A Servo-powered browser runtime as a library, a CLI, and an HTTP server — headless screenshots, scraping, and MCP tools from one Rust binary.',
      url: 'https://hieudoanm.github.io/browsers/',
      codeRepository: 'https://github.com/hieudoanm/browsers',
      offers: { '@type': 'Offer', price: '0', priceCurrency: 'USD' },
    },
  ],
};

interface RootLayoutProps {
  children: ReactNode;
}

const RootLayout: FC<RootLayoutProps> = ({ children }) => {
  const docGroups = [
    {
      label: 'Docs',
      items: getDocPages().map((doc) => ({
        label: doc.title,
        href: doc.slug ? `/docs/${doc.slug}/` : '/docs/',
      })),
    },
  ];

  return (
    <html
      lang="en"
      data-theme="browserverless"
      className={`scroll-smooth antialiased motion-reduce:scroll-auto ${inter.className}`}>
      <body className="bg-base-100 text-base-content m-0 leading-relaxed">
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{ __html: JSON.stringify(JSON_LD) }}
        />
        <Navbar
          name="browserverless"
          logoSrc={LOGO}
          links={NAV_LINKS}
          groups={docGroups}
          githubUrl={GITHUB}
        />
        {children}
        <Footer
          name="browserverless"
          tagline="Servo-powered browser runtime · GPL-3.0 licensed"
          githubUrl={GITHUB}
        />
      </body>
    </html>
  );
};

export default RootLayout;
