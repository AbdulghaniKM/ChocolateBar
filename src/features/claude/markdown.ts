export type Inline = { kind: "text" | "code" | "bold"; text: string };
export type Block =
  | { kind: "paragraph"; inlines: Inline[] }
  | { kind: "heading"; inlines: Inline[] }
  | { kind: "list"; ordered: boolean; items: Inline[][] }
  | { kind: "code"; text: string };

const FENCE = /^```/;
const BULLET = /^\s*[-*•]\s+/;
const NUMBERED = /^\s*\d+[.)]\s+/;
const HEADING = /^#{1,6}\s+/;
const INLINE = /(`[^`]+`|\*\*[^*]+\*\*)/g;

/** A deliberately small Markdown subset (code, bold, lists, headings) rendered as text nodes, never HTML. */
export function parseMarkdown(source: string): Block[] {
  const blocks: Block[] = [];
  const lines = source.replace(/\r\n/g, "\n").split("\n");
  let paragraph: string[] = [];
  let list: { ordered: boolean; items: Inline[][] } | null = null;

  const flushParagraph = () => {
    if (paragraph.length) blocks.push({ kind: "paragraph", inlines: parseInline(paragraph.join(" ")) });
    paragraph = [];
  };
  const flushList = () => {
    if (list) blocks.push({ kind: "list", ...list });
    list = null;
  };

  for (let index = 0; index < lines.length; index++) {
    const line = lines[index];
    if (FENCE.test(line.trim())) {
      flushParagraph();
      flushList();
      const code: string[] = [];
      while (++index < lines.length && !FENCE.test(lines[index].trim())) code.push(lines[index]);
      blocks.push({ kind: "code", text: code.join("\n") });
      continue;
    }
    const ordered = NUMBERED.test(line);
    if (BULLET.test(line) || ordered) {
      flushParagraph();
      if (!list || list.ordered !== ordered) {
        flushList();
        list = { ordered, items: [] };
      }
      list.items.push(parseInline(line.replace(ordered ? NUMBERED : BULLET, "")));
      continue;
    }
    if (!line.trim()) {
      flushParagraph();
      flushList();
      continue;
    }
    flushList();
    if (HEADING.test(line)) {
      flushParagraph();
      blocks.push({ kind: "heading", inlines: parseInline(line.replace(HEADING, "")) });
      continue;
    }
    paragraph.push(line.trim());
  }
  flushParagraph();
  flushList();
  return blocks;
}

export function parseInline(text: string): Inline[] {
  return text
    .split(INLINE)
    .filter(Boolean)
    .map((part) => {
      if (part.startsWith("`") && part.endsWith("`") && part.length > 2) return { kind: "code", text: part.slice(1, -1) };
      if (part.startsWith("**") && part.endsWith("**") && part.length > 4) return { kind: "bold", text: part.slice(2, -2) };
      return { kind: "text", text: part };
    });
}
