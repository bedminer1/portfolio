<script lang="ts">
	/**
	 * Projects — expandable project cards with devlog links.
	 *
	 * Each project has a toggleable description. "devlog →" links route
	 * to /projects/<slug> for per-project update entries.
	 */
	import { projects } from "$lib/projects"
</script>

<section id="my-stuff" class="scroll-mt-8 border-t border-foreground/10 bg-white/25 px-6 py-28 dark:bg-white/[0.015] md:px-10 md:py-36">
	<div class="mx-auto grid max-w-7xl gap-12 md:grid-cols-[0.72fr_1.28fr] md:gap-20">
		<div>
			<h2 class="text-4xl font-semibold tracking-[-0.065em] md:text-5xl">My stuff</h2>
		</div>

	<div class="grid gap-px overflow-hidden border border-foreground/10 bg-foreground/10 xl:grid-cols-2">
		{#each projects as project}
		<a
			href={project.link || `/projects/${project.slug}`}
			target={project.link ? '_blank' : undefined}
			rel={project.link ? 'noreferrer' : undefined}
			class="group block bg-paper p-7 transition duration-300 hover:-translate-y-1 hover:bg-navy/[0.035] hover:shadow-[0_16px_35px_rgba(20,52,103,0.08)] focus-visible:-translate-y-1 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-navy/40 md:p-9"
		>
			<div class="flex items-center justify-between gap-4">
				<div class="flex min-w-0 items-center gap-4">
					{#if project.image}
						<img src={project.image.src} alt={project.image.alt} class="h-16 w-16 shrink-0 rounded-2xl border border-foreground/10 bg-white/50 object-contain shadow-[0_8px_22px_rgba(20,52,103,0.07)] dark:bg-white/[0.04]" />
					{/if}
					<h3 class="text-base font-semibold tracking-[-0.04em] transition group-hover:text-navy">{project.title}</h3>
				</div>
				<span class="shrink-0 text-sm text-navy transition duration-300 group-hover:-translate-y-0.5 group-hover:translate-x-0.5">{project.link ? '↗' : '→'}</span>
			</div>

			<div class="mb-2 mt-5 text-xs leading-6 text-muted-foreground">
				<p>{@html project.description}</p>
				<span class="mt-5 inline-block text-[11px] font-semibold text-navy underline decoration-navy/35 underline-offset-4 transition group-hover:decoration-navy">
					{project.link ? 'live site ↗' : 'devlog →'}
				</span>
			</div>
		</a>
		{/each}
	</div>
	</div>
</section>
