# src/hooks/useInitialLoad.ts

Runs a loading function once when a component mounts. `useInitialLoad(load,
onLoaded, onError)` passes the result to `onLoaded` or the failure to
`onError`, and ignores results that arrive after the component has unmounted.
The three arguments must be stable between renders. Used by the passwords and
files lists.
