# AGENTS.md — packages/landing

Next.js 16 (App Router) static-export landing page + docs site, Tailwind CSS
v4 + DaisyUI 5, deployed to GitHub Pages at
`https://hieudoanm.github.io/browsers/` (project site — `basePath:
"/browsers"`, `assetPrefix` likewise in production).

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
  content passed as props; the Docs dropdown is fed by `getDocPages()`).
- `src/app/page.tsx` — marketing page content passed to the shared components.
- `src/app/docs/[[...slug]]/page.tsx` — docs catch-all; renders every Markdown
  file under `docs/` as a static page at `/browsers/docs/<path>/` with search,
  sectioned sidebar, "On this page" TOC, and prev/next navigation.
- `src/app/api/search-index/route.ts` — `force-static` GET emitting the docs
  search index as JSON for `DocsSearch`.
- `src/components/molecules/` — shared components (`Navbar`, `Hero`,
  `Features`, `CTA`, `Footer`, `SectionHead`, `Terminal`, `DocsSearch`). These
  files are **byte-identical across all `hieudoanm/*/packages/landing` repos** —
  do not fork them; pass content via props instead.
- `src/lib/docs.ts` — Markdown → HTML pipeline (`marked`), front matter
  (`title`, `order`), relative/`.md` and absolute `/docs/…` link rewriting to
  `/browsers/docs/*`, GitHub-style heading ids, TOC and plain-text extraction
  for search. `getDocPages` is the only accessor — keep markdown parsing out
  of components.
- `src/lib/sections.ts` — `DOC_SECTIONS` sidebar groups and `sectionLabel`.
- `src/lib/site.ts` — repository/site URLs and constants.
- `src/app/globals.css` — Tailwind v4 imports + the custom DaisyUI
  `browserverless` light theme.
- `docs/` — documentation source, organized as `<group>/<chapter>.md`.
  Groups: `guide/` (getting-started, cli), `api/` (http-api, mcp),
  `deployment/` (docker, aws, azure, gcp, terraform, heroku, render),
  `project/` (architecture, packaging, roadmap). `README.md` at the root is
  the docs landing page; `order` front matter controls nav order. In-doc
  links may be relative `.md` paths or absolute `/docs/<group>/<chapter>/`
  paths — both are rewritten by `lib/docs.ts`.
- `public/` — `favicon.ico`, `favicon.svg`, `logo.svg`, `og.png`, `robots.txt`,
  `sitemap.xml`, `humans.txt`, served under `/browsers/` in production.
  Canonical sources live in `assets/branding/svg/` — regenerate with
  `assets/svg-to-png.sh`, do not hand-edit the PNGs.

## Conventions

- Keep `output: "export"` in `next.config.ts` — the site must build to static
  HTML in `out/`; the only route handler is the `force-static`
  `api/search-index` GET that emits JSON at build time.
- `basePath`/`assetPrefix` are `/browsers` in production builds;
  `next/link` and metadata get it automatically, raw `<img src>` paths must
  include it (see the `LOGO` constant in `layout.tsx`).
- Theme is light-only via `data-theme="browserverless"` on `<html>`; accent
  colours differ per repo but every theme is light.
- Fonts: Inter via `next/font`; mono stack for terminal/code only.
- Design follows the "AI unslop" rules: no filler badges, monospace only for
  code/commands, real CLI output in demos.
- The landing page markets the CLI only — no app functionality.
- Docs content should stay accurate to `docs/` and `README.md` at the repo
  root; when behaviour changes, update `docs/` in the same change.
