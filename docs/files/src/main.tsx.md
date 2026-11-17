# src/main.tsx

## Purpose

Entry point of the interface: mounts the [`App`](App.tsx.md) component in the `#root`
element of `index.html`.

## Place in the architecture

`index.html` loads this module with `<script type="module" src="/src/main.tsx">`. Vite
bundles it, and Tauri serves the result in the window. It also imports `styles.css`, so the
global styles (light and dark themes) are part of the bundle. Everything else starts from
[`App`](App.tsx.md).

## Public API

None. The file only has a side effect: it renders the app.

## Internal structure

`ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(...)` with
`<App />` inside `React.StrictMode`.

## Dependencies

- Internal: [`App.tsx`](App.tsx.md), `styles.css`.
- External: `react`, `react-dom/client`.

## Usage example

There is nothing to call. It runs when the page loads, with `npm run dev` (interface only),
`npm run tauri dev` or a built app. The root element it needs is in
[`index.html`](../../index.html):

```html
<div id="root"></div>
<script type="module" src="/src/main.tsx"></script>
```

## Edge cases, invariants and error handling

- The element is read with a type assertion (`as HTMLElement`), with no check. If
  `index.html` had no `#root`, `createRoot` would throw at start and the window would stay
  blank.
- In development, `StrictMode` renders components and runs effects twice to expose
  mistakes. It has no effect in production builds.
- The file is excluded from the coverage report in `vite.config.ts`, because it only
  mounts the app.

## Security

Nothing sensitive is handled here. The content security policy that restricts what the page
can load is set in `tauri.conf.json`, not in this file.

## Related files and tests

- Related: [`App.tsx`](App.tsx.md), [`index.html`](../../index.html),
  [`vite-env.d.ts`](vite-env.d.ts.md).
- Tests: none. The mount is covered by every start of the app, and `App` has its own tests
  in [`App.test.tsx`](../../../src/App.test.tsx).
