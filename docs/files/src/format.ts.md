# src/format.ts

## Purpose

Display formatting helpers. For now it has one: turning a byte count into text for people.

## Place in the architecture

[`FileTable`](components/FileTable.tsx.md) uses `formatBytes` for the size column of the file
list. It is a pure function with no dependency on the rest of the interface.

## Public API

`formatBytes(bytes: number): string`

| Input                  | Output       |
| ---------------------- | ------------ |
| `0`                    | `0 bytes`    |
| `1`                    | `1 byte`     |
| `999`                  | `999 bytes`  |
| `1000`                 | `1 KB`       |
| `1500`                 | `1.5 KB`     |
| `2_340_000`            | `2.3 MB`     |
| `150_000_000`          | `150 MB`     |
| `999_500`              | `1 MB`       |
| `5_000_000_000_000_000` | `5000 TB`   |

The units are decimal (1 KB is 1000 bytes), up to terabytes. The function has no side effects
and does not throw.

## Internal structure

- Below 1000 it returns the bytes with the singular `1 byte`.
- Otherwise it divides by 1000 and moves to the next unit while the rounded value reaches
  1000. Comparing the rounded value means `999.9 KB` is shown as `1 MB`, not as `1000 KB`.
- `roundForDisplay` gives whole numbers from 100 up and one decimal below that.
- There is no unit above `TB`, so very large values show a large number of terabytes.

## Dependencies

- Internal: none.
- External: none.

## Usage example

Checked by running the real module with Node:

```ts
import { formatBytes } from "./format";

formatBytes(1500);       // "1.5 KB"
formatBytes(999_499);    // "999 KB"
formatBytes(999_500);    // "1 MB"
```

## Edge cases, invariants and error handling

- The input is expected to be a non-negative integer, as the core sends sizes. Other values
  are not guarded. Running the module gave `formatBytes(-5)` = `-5 bytes`,
  `formatBytes(1.5)` = `1.5 bytes` and `formatBytes(NaN)` = `NaN KB`.
- Decimal units were chosen over binary ones, so sizes match what most file managers show.
- Rounding is to the nearest value, so `999_499` bytes is `999 KB`.

## Security

None. The function only formats a number.

## Related files and tests

- Related: [`components/FileTable.tsx`](components/FileTable.tsx.md),
  [`api/types.ts`](api/types.ts.md) (`FileSummary.size`).
- Tests: [`format.test.ts`](../../../src/format.test.ts) covers small sizes, the decimals,
  moving up a unit at the rounding edge and the terabyte limit.
