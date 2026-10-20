export function formatBytes(value: number): string {
  if (!Number.isFinite(value) || value < 0) return "—";
  if (value < 1_000) return value + " B";
  const units = ["KB", "MB", "GB", "TB"];
  let size = value / 1_000;
  let unitIndex = 0;
  while (size >= 1_000 && unitIndex < units.length - 1) {
    size /= 1_000;
    unitIndex += 1;
  }
  const digits = size >= 100 ? 0 : size >= 10 ? 1 : 2;
  return size.toLocaleString("en-US", { maximumFractionDigits: digits }) + " " + units[unitIndex];
}

export function formatSignedBytes(value: number): string {
  const size = formatBytes(Math.abs(value));
  if (value > 0) return size + " saved";
  if (value < 0) return size + " larger";
  return "No size change";
}
