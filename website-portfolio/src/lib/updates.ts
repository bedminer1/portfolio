import data from './content/notes.json';
import type { ContentBlock } from './content-types';
export interface Note { id: string; title: string; date: string; tags: string; description: string; text?: string; blocks?: ContentBlock[] }
export const notes = data as Note[];
