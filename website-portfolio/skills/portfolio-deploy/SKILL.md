---
name: portfolio-deploy
description: Build, deploy, or maintain Alexander Seah's robotsbybed portfolio and its Telegram content editor on the main Hetzner VPS.
---

# Portfolio deployment

The website source is `website-portfolio` within `/home/alex/Desktop/robotics/PORTFOLIO`. It is a prerendered SvelteKit site. The live URL is `https://robotsbybed.properrobotics.org`, served by Caddy from `/var/www/portfolio` on `root@178.105.168.55` (`ubuntu-4gb-nbg1-1`). Previous deployment records confirm this layout as of October 2026; inspect the server before mutating it.

## Site updates

1. Preserve any existing checkout changes. Review the content JSON in `src/lib/content`, then run `npm run check` and `npm run build`. Verify `build/index.html` and affected project pages.
2. Inspect SSH access, `/var/www/portfolio`, the Caddy site block, and available disk space. The server also runs unrelated services; avoid changing their files or Caddy routes.
3. Back up the existing `/var/www/portfolio` into `/var/backups/portfolio` with a timestamp. Sync only the built static files to `/var/www/portfolio`, using `rsync --delete` only against that dedicated directory. The repository's `scripts/publish.sh` performs the local build, backup, and sync when run on the server with `SITE_ROOT=/var/www/portfolio`.
4. Check that Caddy remains valid and active. Verify the public homepage and any affected project route over HTTPS. A static content update does not require a new port or a Caddy reload.

## Telegram editor

`bot/` contains a Teloxide long-polling bot. It edits JSON files in the same source tree and calls `scripts/publish.sh` after each confirmed change. Photos go to `static/uploads`. Keep the source tree, uploads, and bot service on the VPS so later Telegram updates can rebuild locally there. The service runs as `portfolio-editor.service` from `/opt/portfolio-editor` and is restricted to the owner's Telegram user ID. The service file and setup details are in `bot/portfolio-editor.service` and the site README. `@yap_victim_bot` is the current bot username.

Keep the bot token in `/etc/portfolio-editor.env` with mode 0600. Never put it in Git, skill text, shell history, or command output. `TELEGRAM_OWNER_ID` restricts writes to one user. If resetting the owner, use `/whoami` privately to learn the numeric ID, then update the environment file and restart the service. Validate with a private test note and delete it after checking both site and photo ordering.

If shell networking is blocked, do not claim a deployment. Keep the verified build and exact next step for an environment with SSH access.
