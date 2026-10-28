<script lang="ts">
  import { Marked } from 'marked';
  import type { ContentBlock } from '$lib/content-types';
  let { blocks = [], text = '', image }: { blocks?: ContentBlock[]; text?: string; image?: { src: string; alt: string } } = $props();
  let content = $derived(blocks.length ? blocks : [
    ...(text ? [{ type: 'text' as const, text }] : []),
    ...(image ? [{ type: 'image' as const, ...image }] : [])
  ]);
  const markdown = new Marked({
    gfm: true,
    breaks: true,
    renderer: {
      link({ href, text }) {
        const safe = /^(https?:\/\/|\/)(?!\/)/i.test(href);
        if (!safe) return text;
        const escaped = href.replaceAll('&', '&amp;').replaceAll('"', '&quot;').replaceAll('<', '&lt;');
        return `<a href="${escaped}" rel="noopener noreferrer" target="_blank">${text}</a>`;
      },
      image({ text }) { return text; }
    }
  });
  function render(source: string): string {
    return markdown.parse(source) as string;
  }
</script>

<div class="space-y-4 [&_a]:text-navy [&_a]:underline [&_a]:underline-offset-4 [&_ul]:ml-4 [&_ul]:list-disc [&_ol]:ml-4 [&_ol]:list-decimal">
  {#each content as block}
    {#if block.type === 'text'}
      <div class="prose prose-sm max-w-none text-inherit dark:prose-invert">{@html render(block.text)}</div>
    {:else}
      <figure>
        <img src={block.src} alt={block.alt} loading="lazy" class="max-w-full rounded-md border border-border" />
        {#if block.alt}<figcaption class="mt-1 text-xs italic text-muted-foreground">{block.alt}</figcaption>{/if}
      </figure>
    {/if}
  {/each}
</div>
