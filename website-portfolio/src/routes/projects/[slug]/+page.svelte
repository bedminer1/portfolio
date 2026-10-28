<script lang="ts">
	import ContentBlocks from "$lib/components/content-blocks.svelte";
	import { projects } from "$lib/projects";

	let { data } = $props();

	let project = $derived(projects.find(project => project.slug === data.slug));
	let title = $derived(project?.title ?? data.slug);

	// Accordion state: keyed by entry index
	let expanded: Record<number, boolean> = $state({});

	function toggle(idx: number) {
		expanded[idx] = !expanded[idx];
	}


</script>

<svelte:head>
	<title>{title} — robotsbybed</title>
</svelte:head>

<div class="mx-auto min-h-screen max-w-2xl px-6 py-20 md:py-28">
	<a href="/#my-stuff" class="mb-8 inline-block text-xs text-muted-foreground underline decoration-navy/40 underline-offset-4 hover:text-navy">
		← back to my stuff
	</a>

	<h1 class="text-2xl text-foreground mb-2">{title}</h1>

	<div class="mb-12 h-px bg-foreground/10"></div>

	<section>
		<h2 class="text-sm text-muted-foreground uppercase tracking-widest mb-6">Devlog</h2>

		<div class="space-y-1 max-w-prose">
			{#each data.entries as entry, i}
				<div>
					<button
						onclick={() => toggle(i)}
						class="flex w-full items-baseline justify-between gap-4 text-left bg-transparent border-none cursor-pointer text-muted-foreground hover:text-foreground"
					>
						<span>{entry.title}</span>
						<span class="text-xs shrink-0 tabular-nums opacity-60">
							{entry.date} {expanded[i] ? '▲' : '▼'}
						</span>
					</button>

					{#if expanded[i]}
						<div class="mt-3 mb-8 text-sm leading-relaxed">
							<div class="[&_code]:text-sm text-muted-foreground">
								<ContentBlocks blocks={entry.blocks} text={entry.text} image={entry.image} />
							</div>

							<div class="mt-4 h-px w-1/4 bg-foreground/10"></div>
						</div>
					{/if}
				</div>
			{/each}
		</div>

		{#if data.entries.length === 0}
			<p class="text-sm text-muted-foreground italic">{data.slug === 'robomaster' ? "I'll dump all the good stuff after our competition next apr" : 'No devlog entries yet.'}</p>
		{/if}
	</section>
</div>
