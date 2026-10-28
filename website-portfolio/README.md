# Portfolio

Static SvelteKit site for [robotsbybed.properrobotics.org](https://robotsbybed.properrobotics.org). Run `npm run build` to generate `build/`.

## Edit content

`src/lib/content/main.json` controls the hero copy and metadata. `about.json` controls the name, profile, GitHub link, and bio. `projects.json` defines the visible project cards. `notes.json` and `devlogs.json` hold entries. Entries may have an ordered `blocks` array containing `{ "type": "text", "text": "Markdown" }` and `{ "type": "image", "src": "/uploads/name.jpg", "alt": "Caption" }`. Older entries retain their `text` and optional `image` fields and still render.

## Telegram editor

`bot/` is a Rust Teloxide bot. In a private chat, `/help` lists commands. `/projects` lists usable project slugs such as `calibur`; a slug is the short name in a project page URL. `/list notes` or `/show note` lists note titles and IDs; `/list devlogs calibur` or `/show devlog calibur` does the same for one project. The ID identifies one entry for `/show`, `/edit`, or `/delete`. `/new note` and `/new devlog calibur` guide creation field by field. Send any number of text messages and photos in the body; each becomes a block in the order sent. A photo caption is used as its description. `/done` saves and publishes. `/edit note ID body` or `/edit devlog SLUG ID body` replaces a body using the same flow. `/edit` also supports individual metadata fields. `/list`, `/show`, and `/delete` cover the other entry operations. `/main FIELD` and `/about FIELD` edit the home copy. `/cancel` discards a draft.

The bot accepts mutations only from `TELEGRAM_OWNER_ID` in a private chat. `/whoami` is available before owner setup. Bot state for an unfinished draft is in memory; completed content is in JSON. The provided token appeared in chat, so rotate it with BotFather before production use.

## Main Hetzner VPS

The site and `portfolio-editor.service` are deployed on the VPS. The bot source lives at `/opt/portfolio-editor`; Caddy serves `/var/www/portfolio`. Future content edits through Telegram rebuild and publish automatically. The verified host is `root@178.105.168.55`, with Caddy serving the portfolio from `/var/www/portfolio`. Check the current server state before deployment. `scripts/publish.sh` builds, backs up, and synchronizes the static site when run **on that server** with `SITE_ROOT=/var/www/portfolio`.

For the bot, put this source tree on the server at `/opt/portfolio-editor` without replacing `static/uploads`. Install Node dependencies with `npm ci --legacy-peer-deps`, compile `bot/` with `cargo build --release --manifest-path bot/Cargo.toml`, and ensure `rsync` is installed. Create a dedicated `portfolio-editor` user that owns `/opt/portfolio-editor`, `/var/www/portfolio`, and `/var/backups/portfolio`; the service needs write access to content JSON, uploads, build output, and those two deployment directories. Install `bot/portfolio-editor.service` as `/etc/systemd/system/portfolio-editor.service`.

Create `/etc/portfolio-editor.env` with mode `0600` and these variables:

```text
TELOXIDE_TOKEN=<rotated token>
TELEGRAM_OWNER_ID=0
PORTFOLIO_ROOT=/opt/portfolio-editor
SITE_ROOT=/var/www/portfolio
NPM_CONFIG_CACHE=/tmp/npm-cache
```

Start the service, send `/whoami` privately, set the returned numeric ID in the environment file, then restart it. Test a draft note with text, photo, text, and `/done`; verify the live page, then delete the test note. Keep `static/uploads` and `src/lib/content` in server backups. Never sync an older local copy over the server's content without first pulling the latest server changes.
