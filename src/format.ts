/** Display formatting helpers. */

const UNITS = ["KB", "MB", "GB", "TB"] as const;

/** A byte count for people, in decimal units: "512 bytes", "1.5 MB". */
export function formatBytes(bytes: number): string {
  if (bytes < 1000) {
    return bytes === 1 ? "1 byte" : `${bytes} bytes`;
  }
  let value = bytes / 1000;
  let unit = 0;
  // Compare the rounded value, so 999.9 KB is shown as "1 MB", not "1000 KB".
  while (roundForDisplay(value) >= 1000 && unit < UNITS.length - 1) {
    value /= 1000;
    unit += 1;
  }
  return `${roundForDisplay(value)} ${UNITS[unit]}`;
}

/** Whole numbers from 100 up, one decimal below. */
function roundForDisplay(value: number): number {
  return value >= 100 ? Math.round(value) : Math.round(value * 10) / 10;
}
