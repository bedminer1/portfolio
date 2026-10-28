import type { PageLoad } from "./$types";
import { devlogs } from "$lib/devlogs";
import { projects } from "$lib/projects";

export function entries() {
	return projects.filter(project => !project.link).map(project => ({ slug: project.slug }));
}

export const load: PageLoad = ({ params }) => {
	return {
		slug: params.slug,
		entries: devlogs[params.slug] ?? [],
	};
};
