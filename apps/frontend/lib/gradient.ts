/** Deterministic pastel gradient from a seed — stable across reloads/SSR (no hydration mismatch). */
export function pastelGradient(seed: string): string {
  let hash = 0;
  for (let i = 0; i < seed.length; i++) {
    hash = (hash * 31 + seed.charCodeAt(i)) >>> 0;
  }

  const hue1 = hash % 360;
  const hue2 = (hue1 + 40 + ((hash >> 8) % 60)) % 360;
  const angle = (hash >> 16) % 360;

  return `linear-gradient(${angle}deg, hsl(${hue1} 70% 85%), hsl(${hue2} 65% 78%))`;
}
