<script lang="ts">
	/**
	 * Updates — expandable blog/journal entries.
	 *
	 * Each entry shows title + date. Clicking toggles the full text inline,
	 * rendered from markdown (via `marked`) into HTML. Archived entries are
	 * filtered out. Tags render as shadcn Badge components.
	 */
	import { notes } from "$lib/updates"
	import ContentBlocks from "$lib/components/content-blocks.svelte"

	const visibleNotes = notes.filter(note => !note.tags.split(",").some(tag => tag.trim() === "Archived"))

	/**
	 * Map of note-title → expanded-state.
	 * Re-assigning after mutation signals Svelte to re-render.
	 * @type {Record<string, boolean>}
	 */
	let expanded: Record<string, boolean> = {}

	/** Toggle a note's expanded state by its title key. */
	function toggle(title: string) {
		expanded[title] = !expanded[title]
		expanded = expanded // trigger reactivity
	}

	function displayTags(tags: string): string[] {
		return tags.split(",").filter(tag => tag.trim() !== "Archived")
	}


</script>

<section id="notes" class="scroll-mt-8 mx-auto max-w-7xl px-6 py-28 md:px-10 md:py-36">
	<div class="grid gap-12 md:grid-cols-[0.72fr_1.28fr] md:gap-20">
		<div>
			<h2 class="text-4xl font-semibold tracking-[-0.065em] md:text-5xl">Notes</h2>
		</div>

	<div class="space-y-0 border-t border-foreground/10">
		{#if visibleNotes.length === 0}
			<div class="group border-b border-foreground/10 py-8 transition-all duration-300 hover:border-navy/35 hover:pl-2">
				<p class="text-sm font-medium tracking-[-0.025em] text-foreground">Nothing here yet.</p>
			</div>
		{/if}
		{#each visibleNotes as note}
		<article class="border-b border-foreground/10 py-5">
			<!-- Toggle button -- title left, date + arrow right -->
			<button
				onclick={() => toggle(note.title)}
				class="group flex w-full items-baseline justify-between gap-5 border-none bg-transparent text-left text-muted-foreground transition hover:text-navy"
			>
				<span class="text-sm font-medium tracking-[-0.025em]">{note.title}</span>
				<span class="shrink-0 text-[10px] tabular-nums opacity-60">
					{note.date} <span class="ml-2 text-navy">{expanded[note.title] ? '−' : '+'}</span>
				</span>
			</button>

			{#if expanded[note.title]}
			<div class="mb-3 mt-5 text-xs leading-6 text-muted-foreground">
				<!-- Short summary -->
				<p class="mb-3 italic opacity-70">{note.description}</p>

				<!-- Tag badges -->
				{#if displayTags(note.tags).length > 0}
				<div class="flex gap-2 flex-wrap mb-4">
					{#each displayTags(note.tags) as tag}
						<span class="rounded-full bg-navy/8 px-2.5 py-1 text-[9px] font-semibold uppercase tracking-wider text-navy">{tag}</span>
					{/each}
				</div>
				{/if}

				<!-- Full body rendered from markdown -->
				<div class="[&_code]:text-xs">
					<ContentBlocks blocks={note.blocks} text={note.text} />
				</div>
			</div>
			{/if}
		</article>
		{/each}
	</div>
	</div>
</section>
