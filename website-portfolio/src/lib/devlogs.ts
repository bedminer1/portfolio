import data from './content/devlogs.json';
import type { ContentBlock } from './content-types';
export interface DevlogEntry { id: string; title: string; date: string; text?: string; blocks?: ContentBlock[]; image?: {src: string; alt: string} }
export const devlogs = data as Record<string, DevlogEntry[]>;
