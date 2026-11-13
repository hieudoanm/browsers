# @browsers/landing

Landing page + documentation site for browserverless — Next.js 16 static
export deployed to GitHub Pages.

## Structure

```text
src/app/             # routes: / (marketing), /docs, /docs/[...slug]
src/components/      # shared UI (molecules are shared across hieudoanm landings)
src/lib/             # site constants + markdown doc loading (md/docs/)
md/docs/             # documentation source, grouped <group>/<chapter>.md
public/              # favicon, logo, og.png, robots, sitemap, humans
```

Docs are plain markdown files in `md/docs/` with `title`/`description`/`order`
front matter; `src/lib/docs.ts` turns them into static routes at
`/docs/<group>/<chapter>/`.

## Commands

```bash
pnpm install
pnpm run dev       # dev server (no basePath)
pnpm run build     # static export -> out/ (basePath /browsers in prod)
pnpm run lint
pnpm run typecheck
pnpm run format
```

## Deployment

CI builds this package and deploys `out/` to GitHub Pages — the site lives at
`https://hieudoanm.github.io/browsers/`.
