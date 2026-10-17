<!-- Guide for AI working in the React UI (src/). -->
# src (UI)

React 19 + TypeScript + Tailwind v4 + Motion + ECharts + zustand. Design rules: `../docs/SPEC.md` §4.
- `lib/`: `api.ts` (the only backend access), `types.ts` (mirror of Rust types), `format.ts`.
- `components/`: shared design-system components (re-exported from `components/index.ts`); props in CONTRACT.md.
- `shell/`: app frame (sidebar, menu bar, dialogs, shortcuts, theme). `pages/<id>/index.tsx`: one folder per page.
- `store/`: `ui.ts`, `settings.ts`, `scan.ts` (zustand).
- Colours come from the CSS variables in `styles/global.css`. Never hard-code hex values in components
  (category colours come from `categoryMeta`).
