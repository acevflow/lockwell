/** Display formatting helpers. */

const UNITS = ["KB", "MB", "GB", "TB"] as const;

/** A byte count for people, in decimal units: "512 bytes", "1.5 MB". */
export function formatBytes(bytes: number): string {
  if (bytes < 1000) {
    return bytes === 1 ? "1 byte" : `${bytes} bytes`;
  }
  let value = bytes;
  let unit = 0;
  value /= 1000;
  while (value >= 1000 && unit < UNITS.length - 1) {
    value /= 1000;
    unit += 1;
  }
  const rounded =
    value >= 100 ? Math.round(value) : Math.round(value * 10) / 10;
  return `${rounded} ${UNITS[unit]}`;
}
