<script lang="ts">
  import about from "$lib/content/about.json";
</script>

<section id="about" class="scroll-mt-16">
	<!-- Header row: name + circular profile photo -->
	<div class="flex items-start gap-8 mb-8">
		<div class="flex-1">
			<h1 class="text-2xl mb-1">{about.name}</h1>
			<a target="_blank" href={about.githubUrl} class="text-muted-foreground underline hover:text-foreground">
				{about.githubHandle}
			</a>
		</div>
		<img class="w-20 h-20 rounded-full object-cover shrink-0 mr-4" src={about.profileImage} alt="profile">
	</div>

	<!-- One-liner bio -- concise, no stack detail dumps -->
	<div class="max-w-prose text-muted-foreground">
		<p>{about.body}</p>
	</div>
</section>
