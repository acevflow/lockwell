# src/format.ts

Display helpers. `formatBytes()` shows a byte count in decimal units
("512 bytes", "1.5 MB"), with one decimal below 100 and up to terabytes. A value that rounds to 1000
moves up a unit ("999 999 bytes" is "1 MB", not "1000 KB").
