const AVATAR_HUES = [24, 36, 150, 200, 230, 265, 330, 8];

export function avatarStyle(name: string): string {
  const hash = [...name].reduce((total, character) => (total * 31 + character.charCodeAt(0)) >>> 0, 7);
  const hue = AVATAR_HUES[hash % AVATAR_HUES.length];
  return `background: hsl(${hue} 45% 26%); color: hsl(${hue} 80% 82%);`;
}

export function initials(name: string): string {
  const words = name.replace(/[^\p{L}\p{N} ]/gu, " ").trim().split(/\s+/);
  const letters = words.length > 1 ? words[0][0] + words[1][0] : (words[0] ?? "?").slice(0, 2);
  return letters.toUpperCase();
}
