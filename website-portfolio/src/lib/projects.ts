import data from './content/projects.json';
export interface Prject { title: string; slug: string; description: string; link: string; image?: {src: string; alt: string} }
export const projects = data as Project[];