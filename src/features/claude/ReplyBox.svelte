<script lang="ts">
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import { setKeyboardInput, sourceCommand } from "../../lib/ipc";

  let { sessionId }: { sessionId: string } = $props();

  const NOTE_VISIBLE_MS = 6000;
  const MAX_ROWS_REM = 6;
  let text = $state("");
  let note = $state("");
  let failed = $state(false);
  let sending = $state(false);
  let field = $state<HTMLTextAreaElement>();
  let noteTimer: ReturnType<typeof setTimeout> | undefined;

  function grow() {
    if (!field) return;
    field.style.height = "auto";
    field.style.height = `min(${field.scrollHeight}px, ${MAX_ROWS_REM}rem)`;
  }

  function show(message: string, isError = false) {
    note = message;
    failed = isError;
    clearTimeout(noteTimer);
    noteTimer = setTimeout(() => (note = ""), NOTE_VISIBLE_MS);
  }

  async function send() {
    const message = text.trim();
    if (!message || sending) return;
    sending = true;
    try {
      const { window } = await sourceCommand<{ window: string | null }>("claude", "reply", { sessionId, text: message });
      text = "";
      grow();
      field?.blur();
      show(window ? "Your message is copied and VS Code is in front. Press Ctrl+V, then Enter." : "Copied. Paste it into Claude Code with Ctrl+V.");
    } catch (error) {
      show(String(error), true);
    } finally {
      sending = false;
    }
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.shiftKey && !event.isComposing) {
      event.preventDefault();
      void send();
    } else if (event.key === "Escape") {
      field?.blur();
    }
  }
</script>

<div class="flex flex-col gap-1">
  <div class="flex items-end gap-2 rounded-xl border border-island-border bg-island-raised/70 py-1.5 pr-1.5 pl-3 transition-colors focus-within:border-accent/60">
    <label class="sr-only" for="claude-reply-{sessionId}">Reply to Claude</label>
    <textarea
      bind:this={field}
      bind:value={text}
      id="claude-reply-{sessionId}"
      class="max-h-24 min-h-6 flex-1 resize-none bg-transparent py-0.5 text-[0.8125rem] leading-snug text-island-text outline-none placeholder:text-island-subtle"
      rows="1"
      placeholder="Reply to Claude…"
      dir="auto"
      onpointerdown={() => void setKeyboardInput(true)}
      onfocus={() => void setKeyboardInput(true)}
      onblur={() => void setKeyboardInput(false)}
      oninput={grow}
      onkeydown={onKeydown}
    ></textarea>
    <button
      class="grid size-7 shrink-0 place-items-center rounded-lg bg-accent text-on-accent transition-[background-color,opacity] hover:bg-accent-strong disabled:opacity-30"
      type="button"
      aria-label="Send to Claude Code"
      disabled={!text.trim() || sending}
      onclick={() => void send()}
    >
      <ArrowUp class="size-4" strokeWidth={2.5} />
    </button>
  </div>
  {#if note}
    <p class="px-1 text-[0.6875rem] {failed ? 'text-danger' : 'text-island-muted'}" role="status">{note}</p>
  {/if}
</div>
