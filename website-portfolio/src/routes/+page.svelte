<script lang="ts">
  import { onMount } from 'svelte';
  import MyStuff from '$lib/components/projects.svelte';
  import Notes from '$lib/components/updates.svelte';
  import main from '$lib/content/main.json';
  import about from '$lib/content/about.json';

  let dark = false;

  onMount(() => {
    dark = localStorage.getItem('portfolio-theme') === 'dark';
    document.documentElement.classList.toggle('dark', dark);
  });

  function toggleTheme() {
    dark = !dark;
    document.documentElement.classList.toggle('dark', dark);
    localStorage.setItem('portfolio-theme', dark ? 'dark' : 'light');
    document.querySelector<HTMLMetaElement>('#theme-color')?.setAttribute('content', dark ? '#0c1627' : '#fcfaf5');
  }
</script>

<svelte:head>
  <title>{main.pageTitle}</title>
  <meta name="description" content="{main.pageDescription}" />
</svelte:head>

<div class="min-h-screen overflow-hidden">
  <header class="mx-auto flex max-w-7xl items-center justify-between px-6 py-7 md:px-10">
    <nav class="flex items-center gap-5 text-[11px] text-muted-foreground md:gap-7" aria-label="Page sections">
      <a class="relative transition after:absolute after:-bottom-1 after:left-0 after:h-px after:w-0 after:bg-navy after:transition-all hover:text-navy hover:after:w-full" href="#my-stuff">my stuff</a>
      <a class="relative transition after:absolute after:-bottom-1 after:left-0 after:h-px after:w-0 after:bg-navy after:transition-all hover:text-navy hover:after:w-full" href="#notes">notes</a>
    </nav>
    <div>
      <button type="button" onclick={toggleTheme} aria-label={dark ? 'Switch to light mode' : 'Switch to dark mode'} class="grid h-9 w-9 place-items-center rounded-full border border-foreground/15 text-sm transition hover:rotate-12 hover:border-navy hover:bg-navy hover:text-paper">
        {dark ? '☼' : '◐'}
      </button>
    </div>
  </header>

  <main id="top">
    <section class="relative mx-auto max-w-7xl px-6 pb-24 pt-12 md:px-10 md:pb-28 md:pt-20">
      <div class="relative z-10 grid items-center gap-14 lg:grid-cols-[minmax(0,0.82fr)_minmax(28rem,1.18fr)] lg:gap-12 xl:gap-20">
        <div class="min-w-0">
          <div class="mb-9 flex items-center gap-5">
            <a href="{main.telegramUrl}" target="_blank" rel="noreferrer" aria-label="Message Alexander on Telegram" class="group relative shrink-0 rounded-full outline-none">
              <span class="watercolor absolute -inset-1.5 opacity-55 transition group-hover:scale-105 group-focus-visible:scale-105"></span>
              <img src={about.profileImage} alt="Alexander Seah diving" class="relative h-16 w-16 rounded-full border-4 border-paper object-cover shadow-[0_10px_28px_rgba(20,52,103,0.14)] transition duration-300 group-hover:-translate-y-1 group-hover:scale-105 group-hover:-rotate-2 group-focus-visible:ring-2 group-focus-visible:ring-navy/35 sm:h-20 sm:w-20" />
              <span class="pointer-events-none absolute left-0 top-full z-20 mt-3 translate-y-1 whitespace-nowrap rounded-full bg-foreground px-3 py-2 text-[10px] font-semibold text-background opacity-0 shadow-lg transition duration-200 group-hover:translate-y-0 group-hover:opacity-100 group-focus-visible:translate-y-0 group-focus-visible:opacity-100">Open Telegram ↗</span>
            </a>
            <div class="min-w-0">
              <h1 class="text-2xl font-medium tracking-[-0.04em]">{about.name}</h1>
              <a href="{main.githubUrl}" target="_blank" rel="noreferrer" class="group relative mt-1 inline-block text-sm text-muted-foreground underline decoration-navy/35 underline-offset-4 transition hover:text-navy hover:decoration-navy">
                {about.githubHandle} ↗
                <span class="pointer-events-none absolute left-0 top-full z-20 mt-3 translate-y-1 whitespace-nowrap rounded-full bg-foreground px-3 py-2 text-[10px] font-semibold text-background opacity-0 shadow-lg transition duration-200 group-hover:translate-y-0 group-hover:opacity-100 group-focus-visible:translate-y-0 group-focus-visible:opacity-100">Open GitHub ↗</span>
              </a>
            </div>
          </div>

          <div class="max-w-xl space-y-3 text-sm leading-7 text-muted-foreground">
            <p>{main.intro}</p>
            <p>{about.body}</p>
          </div>
          <blockquote class="mt-10 max-w-xl border-l-2 border-navy/50 pl-5 text-sm italic leading-8 text-muted-foreground">
            {main.quote}
          </blockquote>
        </div>

        <figure class="mx-auto w-full max-w-2xl lg:max-w-none">
          <img src={main.heroImage} alt="Watercolor RoboMaster sentry robot number 3 with blue lights" class="block h-auto w-full object-contain" />
        </figure>
      </div>
    </section>

    <MyStuff />
    <Notes />
  </main>

  <footer class="border-t border-foreground/10 px-6 py-10 text-center text-[11px]">
    <a class="font-semibold text-navy underline decoration-navy/40 underline-offset-4 transition hover:decoration-navy" href="{main.telegramUrl}" target="_blank" rel="noreferrer">Telegram {main.telegramHandle} ↗</a>
  </footer>
</div>
