# AGENTS.md — packages/landing

Next.js 16 (App Router) static-export landing page + docs site, Tailwind CSS
v4 + DaisyUI 5, deployed to GitHub Pages at
`https://hieudoanm.github.io/browserverless/` (project site — `basePath:
"/browserverless"`, `assetPrefix` likewise in production).

## Commands

```bash
pnpm install
pnpm run dev      # dev server (basePath applies)
pnpm run build    # static export → out/ (uploaded to Pages by CI)
pnpm run lint
pnpm run typecheck
```

## Structure

- `src/app/layout.tsx` — metadata, fonts, and the global `Navbar`/`Footer` (all
  content passed as props; the Docs dropdown is fed by `getDocs()`).
- `src/app/page.tsx` — marketing page content passed to the shared components.
- `src/app/docs/` — docs index (`page.tsx`), sidebar layout (`layout.tsx`), and
  per-page route (`[slug]/page.tsx`) with prev/next navigation.
- `src/components/molecules/` — shared components (`Navbar`, `Hero`,
  `Features`, `CTA`, `Footer`, `SectionHead`, `Terminal`). These files are
  **byte-identical across all `hieudoanm/*/packages/landing` repos** — do not
  fork them; pass content via props instead.
- `src/components/shared/Markdown.tsx` — `react-markdown` + `remark-gfm`
  renderer with prose styling; the only markdown pipeline.
- `src/lib/docs.ts` — recursively loads `md/docs/**/*.md` (front matter:
  `title`, `description`, `order`) into `Doc` records; the slug is the path
  relative to `md/docs/` without `.md` (e.g. `guide/getting-started`), and the
  first segment becomes `doc.group` for sidebar grouping. `getDoc`/`getDocs`
  are the only accessors — keep markdown parsing out of components.
- `src/lib/site.ts` — repository/site URLs and constants.
- `src/app/globals.css` — Tailwind v4 imports + the custom DaisyUI
  `browserverless` light theme.
- `md/docs/` — documentation source, organized as `<group>/<chapter>.md`.
  Groups: `guide/` (getting-started, cli), `api/` (http-api, mcp),
  `project/` (architecture, packaging, roadmap). `index.md` at the root is
  the docs landing page; `order` front matter controls nav order. In-doc
  links must be absolute (`/docs/<group>/<chapter>/`) — relative links break
  under nested slugs.
- `public/` — `favicon.ico`, `favicon.svg`, `logo.svg`, `og.png`, `robots.txt`,
  `sitemap.xml`, `humans.txt`, served under `/browserverless/` in production.
  Canonical sources live in `assets/branding/svg/` — regenerate with
  `assets/svg-to-png.sh`, do not hand-edit the PNGs.

## Conventions

- Keep `output: "export"` in `next.config.ts` — the site must build to static
  HTML in `out/`; no server features or API routes.
- `basePath`/`assetPrefix` are `/browserverless` in production builds;
  `next/link` and metadata get it automatically, raw `<img src>` paths must
  include it (see the `LOGO` constant in `layout.tsx`).
- Theme is light-only via `data-theme="browserverless"` on `<html>`; accent
  colours differ per repo but every theme is light.
- Fonts: Inter via `next/font`; mono stack for terminal/code only.
- Design follows the "AI unslop" rules: no filler badges, monospace only for
  code/commands, real CLI output in demos.
- The landing page markets the CLI only — no app functionality, no API routes.
- Docs content should stay accurate to `docs/` and `README.md` at the repo
  root; when behaviour changes, update `md/docs/` in the same change.
