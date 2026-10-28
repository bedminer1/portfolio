use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use teloxide::{net::Download, prelude::*, types::Message};
use tokio::sync::Mutex;

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum Block {
    Text { text: String },
    Image { src: String, alt: String },
}

#[derive(Clone)]
enum Kind {
    Note,
    Devlog(String),
}
#[derive(Clone)]
struct Draft {
    kind: Kind,
    id: String,
    title: String,
    date: String,
    tags: String,
    description: String,
    blocks: Vec<Block>,
    step: u8,
    editing: bool,
}
#[derive(Clone)]
enum Session {
    Draft(Draft),
    Field {
        kind: String,
        slug: String,
        id: String,
        field: String,
    },
    Confirm {
        kind: Kind,
        id: String,
    },
}
struct App {
    root: PathBuf,
    owner: i64,
    sessions: Mutex<HashMap<i64, Session>>,
    publish_lock: Mutex<()>,
    processing: Mutex<()>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let root = PathBuf::from(env::var("PORTFOLIO_ROOT").context("PORTFOLIO_ROOT is required")?);
    let owner = env::var("TELEGRAM_OWNER_ID")
        .context("TELEGRAM_OWNER_ID is required")?
        .parse()?;
    let token = env::var("TELOXIDE_TOKEN").context("TELOXIDE_TOKEN is required")?;
    let app = Arc::new(App {
        root,
        owner,
        sessions: Mutex::new(HashMap::new()),
        publish_lock: Mutex::new(()),
        processing: Mutex::new(()),
    });
    let bot = Bot::new(token);
    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let app = app.clone();
        async move {
            if let Err(error) = handle(&bot, &msg, &app).await {
                eprintln!("request failed: {error:#}");
                let _ = bot
                    .send_message(msg.chat.id, format!("Could not complete that: {error:#}"))
                    .await;
            }
            respond(())
        }
    })
    .await;
    Ok(())
}

async fn handle(bot: &Bot, msg: &Message, app: &Arc<App>) -> Result<()> {
    if !msg.chat.is_private() {
        return Ok(());
    }
    let chat = msg.chat.id;
    let id = msg
        .from
        .as_ref()
        .map(|user| user.id.0 as i64)
        .unwrap_or_default();
    if msg.text() == Some("/whoami") {
        bot.send_message(chat, format!("Your Telegram user ID: {id}"))
            .await?;
        return Ok(());
    }
    if id != app.owner {
        return Ok(());
    }
    let _processing = app.processing.lock().await;
    let input = msg.text().unwrap_or("").trim();
    if input == "/start" || input == "/help" {
        bot.send_message(chat, "Portfolio editor\n\nA project SLUG is its short URL name. Send /projects to see available slugs (for example, calibur or zeri).\nAn entry ID identifies one note or devlog. Send /list notes or /list devlogs calibur to see titles and IDs.\n\n/new note — guided creation\n/new devlog calibur — guided creation\n/show note — list notes; add an ID to read one\n/show devlog calibur — list that project's devlogs; add an ID to read one\n/edit note ID FIELD or /edit devlog SLUG ID FIELD\n/delete note ID or /delete devlog SLUG ID\n/main FIELD or /about FIELD — edit page copy\n/publish — rebuild and deploy\n/cancel — discard current draft\n\nNote fields: title, date, tags, description, body. Devlog fields: title, date, body. Main fields: intro, quote, pageTitle, pageDescription, telegramHandle, telegramUrl. About fields: name, body, githubHandle, githubUrl.\n\nFor a body, send text and photos in order, then /done. A photo caption becomes its description.").await?;
        return Ok(());
    }
    if input == "/cancel" {
        if let Some(Session::Draft(draft)) = app.sessions.lock().await.remove(&id) {
            cleanup_draft(app, &draft);
        }
        bot.send_message(chat, "Cancelled.").await?;
        return Ok(());
    }
    let command = input.split_whitespace().collect::<Vec<_>>();
    if matches!(
        command.first(),
        Some(
            &"/new"
                | &"/list"
                | &"/show"
                | &"/edit"
                | &"/delete"
                | &"/main"
                | &"/about"
                | &"/publish"
                | &"/projects"
        )
    ) {
        if let Some(Session::Draft(draft)) = app.sessions.lock().await.remove(&id) {
            cleanup_draft(app, &draft);
        }
        match command[0] {
            "/projects" => {
                send_chunks(bot, chat, project_list(app)?).await?;
            }
            "/new" => {
                if command.get(1).copied() == Some("devlog") && command.get(2).is_none() {
                    send_chunks(bot, chat, project_list(app)?).await?;
                    return Ok(());
                }
                let kind = parse_kind(&command, 1)?;
                let draft = Draft {
                    kind,
                    id: format!(
                        "entry-{}",
                        SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis()
                    ),
                    title: String::new(),
                    date: String::new(),
                    tags: String::new(),
                    description: String::new(),
                    blocks: vec![],
                    step: 0,
                    editing: false,
                };
                app.sessions.lock().await.insert(id, Session::Draft(draft));
                bot.send_message(chat, "Title?").await?;
            }
            "/list" => {
                if command.len() == 1 {
                    bot.send_message(chat, "Use /list notes or /list devlogs calibur. Send /projects for the project slugs.").await?;
                    return Ok(());
                }
                if matches!(command.get(1), Some(&"devlog" | &"devlogs"))
                    && command.get(2).is_none()
                {
                    send_chunks(bot, chat, project_list(app)?).await?;
                    return Ok(());
                }
                let kind = parse_kind(&command, 1)?;
                send_chunks(bot, chat, entry_list(app, &kind)?).await?;
            }
            "/show" => {
                if command.len() == 1 {
                    bot.send_message(chat, "Use /show note to list notes, or /show devlog calibur to list a project's devlogs. Send /projects for project slugs.").await?;
                    return Ok(());
                }
                if matches!(command.get(1), Some(&"devlog" | &"devlogs"))
                    && command.get(2).is_none()
                {
                    send_chunks(bot, chat, project_list(app)?).await?;
                    return Ok(());
                }
                let kind = parse_kind(&command, 1)?;
                let id_index = if matches!(kind, Kind::Note) { 2 } else { 3 };
                if command.get(id_index).is_none() {
                    send_chunks(bot, chat, entry_list(app, &kind)?).await?;
                    return Ok(());
                }
                let (kind, entry_id, _) = entry_args(&command, false)?;
                let entry = list_entries(app, &kind)?
                    .into_iter()
                    .find(|v| v["id"] == entry_id)
                    .ok_or_else(|| anyhow!("Entry not found"))?;
                send_chunks(bot, chat, format_entry(&kind, &entry)).await?;
            }
            "/edit" => {
                if command.len() == 1 {
                    bot.send_message(chat, "Use /edit note ID FIELD or /edit devlog SLUG ID FIELD. Send /list notes or /projects to find the names.").await?;
                    return Ok(());
                }
                if matches!(command.get(1), Some(&"devlog" | &"devlogs"))
                    && command.get(2).is_none()
                {
                    send_chunks(bot, chat, project_list(app)?).await?;
                    return Ok(());
                }
                let target = parse_kind(&command, 1)?;
                let id_index = if matches!(target, Kind::Note) { 2 } else { 3 };
                if command.get(id_index).is_none() {
                    send_chunks(bot, chat, entry_list(app, &target)?).await?;
                    return Ok(());
                }
                if command.get(id_index + 1).is_none() {
                    bot.send_message(chat, "Add a field to edit: title, date, tags, description, or body for notes; title, date, or body for devlogs.").await?;
                    return Ok(());
                }
                let (kind, entry_id, field) = entry_args(&command, true)?;
                let allowed = match &kind {
                    Kind::Note => ["title", "date", "tags", "description", "body"].as_slice(),
                    Kind::Devlog(_) => ["title", "date", "body"].as_slice(),
                };
                if !allowed.contains(&field.as_str()) {
                    return Err(anyhow!("Unknown field. Try /help"));
                }
                let entry = list_entries(app, &kind)?
                    .into_iter()
                    .find(|v| v["id"] == entry_id)
                    .ok_or_else(|| anyhow!("Entry not found"))?;
                if field == "body" {
                    let mut draft = Draft {
                        kind,
                        id: entry_id,
                        title: entry["title"].as_str().unwrap_or("").into(),
                        date: entry["date"].as_str().unwrap_or("").into(),
                        tags: entry["tags"].as_str().unwrap_or("").into(),
                        description: entry["description"].as_str().unwrap_or("").into(),
                        blocks: vec![],
                        step: 3,
                        editing: true,
                    };
                    if let Kind::Note = draft.kind {
                        draft.step = 5;
                    }
                    app.sessions.lock().await.insert(id, Session::Draft(draft));
                    bot.send_message(chat, "Send the replacement body as text and photos in order. /done when finished.").await?;
                } else {
                    app.sessions.lock().await.insert(
                        id,
                        Session::Field {
                            kind: match kind {
                                Kind::Note => "note".into(),
                                Kind::Devlog(_) => "devlog".into(),
                            },
                            slug: match &command[1] {
                                &"devlog" | &"devlogs" => command[2].into(),
                                _ => String::new(),
                            },
                            id: entry_id,
                            field: field.clone(),
                        },
                    );
                    bot.send_message(chat, format!("Send the new {field}."))
                        .await?;
                }
            }
            "/delete" => {
                if command.len() == 1 {
                    bot.send_message(chat, "Use /delete note ID or /delete devlog SLUG ID. Send /list notes or /projects to find them.").await?;
                    return Ok(());
                }
                if matches!(command.get(1), Some(&"devlog" | &"devlogs"))
                    && command.get(2).is_none()
                {
                    send_chunks(bot, chat, project_list(app)?).await?;
                    return Ok(());
                }
                let target = parse_kind(&command, 1)?;
                let id_index = if matches!(target, Kind::Note) { 2 } else { 3 };
                if command.get(id_index).is_none() {
                    send_chunks(bot, chat, entry_list(app, &target)?).await?;
                    return Ok(());
                }
                let (kind, entry_id, _) = entry_args(&command, false)?;
                if !list_entries(app, &kind)?
                    .iter()
                    .any(|v| v["id"] == entry_id)
                {
                    return Err(anyhow!("Entry not found"));
                }
                app.sessions
                    .lock()
                    .await
                    .insert(id, Session::Confirm { kind, id: entry_id });
                bot.send_message(chat, "Reply DELETE to confirm, or /cancel.")
                    .await?;
            }
            "/main" | "/about" => {
                let field = command
                    .get(1)
                    .ok_or_else(|| anyhow!("Specify a field. See /help"))?
                    .to_string();
                let allowed = if command[0] == "/main" {
                    [
                        "intro",
                        "quote",
                        "pageTitle",
                        "pageDescription",
                        "telegramHandle",
                        "telegramUrl",
                    ]
                    .as_slice()
                } else {
                    ["name", "body", "githubHandle", "githubUrl"].as_slice()
                };
                if !allowed.contains(&field.as_str()) {
                    return Err(anyhow!("Unknown field. See /help"));
                }
                app.sessions.lock().await.insert(
                    id,
                    Session::Field {
                        kind: command[0][1..].into(),
                        slug: String::new(),
                        id: String::new(),
                        field: field.clone(),
                    },
                );
                bot.send_message(chat, format!("Send the new {field}."))
                    .await?;
            }
            "/publish" => {
                publish(app).await?;
                bot.send_message(chat, "Site published.").await?;
            }
            _ => {}
        }
        return Ok(());
    }
    let session = app.sessions.lock().await.remove(&id);
    match session {
        None => {
            bot.send_message(chat, "Send /help to see the editor commands.")
                .await?;
        }
        Some(Session::Confirm { kind, id: entry_id }) => {
            if input != "DELETE" {
                bot.send_message(chat, "Deletion cancelled.").await?;
                return Ok(());
            }
            mutate_entries(app, &kind, |entries| {
                entries.retain(|v| v["id"] != entry_id);
                Ok(())
            })
            .await?;
            bot.send_message(chat, "Deleted and published.").await?;
        }
        Some(Session::Field {
            kind,
            slug,
            id: entry_id,
            field,
        }) => {
            if input.is_empty() {
                app.sessions.lock().await.insert(
                    id,
                    Session::Field {
                        kind,
                        slug,
                        id: entry_id,
                        field,
                    },
                );
                return Err(anyhow!("Send text for this field"));
            }
            if kind == "main" || kind == "about" {
                let path = app.root.join(format!("src/lib/content/{kind}.json"));
                mutate_file(app, &path, |v| {
                    v[&field] = json!(input);
                    Ok(())
                })
                .await?;
            } else {
                let entry_kind = if kind == "note" {
                    Kind::Note
                } else {
                    Kind::Devlog(slug)
                };
                mutate_entries(app, &entry_kind, |entries| {
                    let entry = entries
                        .iter_mut()
                        .find(|v| v["id"] == entry_id)
                        .ok_or_else(|| anyhow!("Entry not found"))?;
                    entry[&field] = json!(input);
                    Ok(())
                })
                .await?;
            }
            bot.send_message(chat, "Updated and published.").await?;
        }
        Some(Session::Draft(mut draft)) => {
            let body_step = if matches!(draft.kind, Kind::Note) {
                5
            } else {
                2
            };
            if draft.step < body_step {
                if input.is_empty() {
                    app.sessions.lock().await.insert(id, Session::Draft(draft));
                    return Err(anyhow!("Send text for this field"));
                }
                let prompt = match draft.step {
                    0 => {
                        draft.title = input.into();
                        "Date? (e.g. 2026-10-17)"
                    }
                    1 => {
                        draft.date = input.into();
                        if matches!(draft.kind, Kind::Note) {
                            "Tags? Send '-' for none."
                        } else {
                            "Send body text or photos. /done to publish."
                        }
                    }
                    2 => {
                        draft.tags = if input == "-" {
                            String::new()
                        } else {
                            input.into()
                        };
                        "Short description? Send '-' for none."
                    }
                    _ => {
                        draft.description = if input == "-" {
                            String::new()
                        } else {
                            input.into()
                        };
                        "Send body text or photos. /done to publish."
                    }
                };
                draft.step += 1;
                // Note body starts after title/date/tags/description.
                if matches!(draft.kind, Kind::Note) && draft.step == 4 {
                    draft.step = 5;
                }
                app.sessions.lock().await.insert(id, Session::Draft(draft));
                bot.send_message(chat, prompt).await?;
            } else if input == "/done" {
                if draft.blocks.is_empty() {
                    app.sessions.lock().await.insert(id, Session::Draft(draft));
                    return Err(anyhow!("Send at least one body block"));
                }
                let value = draft_value(&draft)?;
                let edit = draft.editing;
                let entry_id = draft.id.clone();
                let saved = mutate_entries(app, &draft.kind, |entries| {
                    if edit {
                        let entry = entries
                            .iter_mut()
                            .find(|v| v["id"] == entry_id)
                            .ok_or_else(|| anyhow!("Entry not found"))?;
                        entry["blocks"] = value["blocks"].clone();
                        entry.as_object_mut().unwrap().remove("text");
                        entry.as_object_mut().unwrap().remove("image");
                    } else {
                        entries.insert(0, value);
                    }
                    Ok(())
                })
                .await;
                if let Err(error) = saved {
                    app.sessions.lock().await.insert(id, Session::Draft(draft));
                    return Err(error);
                }
                bot.send_message(chat, "Saved and published.").await?;
            } else if let Some(photo) = msg.photo().and_then(|sizes| sizes.last()) {
                if photo.file.size > 10_000_000 {
                    app.sessions.lock().await.insert(id, Session::Draft(draft));
                    return Err(anyhow!("Photo exceeds 10 MB"));
                }
                let remote = bot.get_file(photo.file.id.clone()).await?;
                let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
                let name = format!("{}-{stamp}.jpg", draft.id);
                let path = app.root.join("static/uploads").join(&name);
                fs::create_dir_all(path.parent().unwrap())?;
                let mut file = tokio::fs::File::create(&path).await?;
                bot.download_file(&remote.path, &mut file).await?;
                draft.blocks.push(Block::Image {
                    src: format!("/uploads/{name}"),
                    alt: msg.caption().unwrap_or("").into(),
                });
                app.sessions.lock().await.insert(id, Session::Draft(draft));
                bot.send_message(chat, "Photo added. Send more text/photos, or /done.")
                    .await?;
            } else if !input.is_empty() {
                // Escape raw HTML before markdown rendering.
                draft.blocks.push(Block::Text {
                    text: input
                        .replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;"),
                });
                app.sessions.lock().await.insert(id, Session::Draft(draft));
                bot.send_message(chat, "Text added. Send more text/photos, or /done.")
                    .await?;
            } else {
                app.sessions.lock().await.insert(id, Session::Draft(draft));
                bot.send_message(chat, "Send text, a photo, or /done.")
                    .await?;
            }
        }
    }
    Ok(())
}

fn parse_kind(args: &[&str], index: usize) -> Result<Kind> {
    match args.get(index).copied() {
        Some("note" | "notes") => Ok(Kind::Note),
        Some("devlog" | "devlogs") => Ok(Kind::Devlog(
            args.get(index + 1)
                .ok_or_else(|| anyhow!("Specify project slug"))?
                .to_string(),
        )),
        _ => Err(anyhow!("Use note or devlog SLUG")),
    }
}
fn entry_args(args: &[&str], field: bool) -> Result<(Kind, String, String)> {
    let kind = parse_kind(args, 1)?;
    let offset = if matches!(kind, Kind::Note) { 2 } else { 3 };
    let id = args
        .get(offset)
        .ok_or_else(|| anyhow!("Specify entry ID"))?
        .to_string();
    let field_name = if field {
        args.get(offset + 1)
            .ok_or_else(|| anyhow!("Specify field"))?
            .to_string()
    } else {
        String::new()
    };
    Ok((kind, id, field_name))
}
fn content_path(app: &App, kind: &Kind) -> Result<PathBuf> {
    match kind {
        Kind::Note => Ok(app.root.join("src/lib/content/notes.json")),
        Kind::Devlog(slug) => {
            if !slug
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            {
                return Err(anyhow!("Invalid project slug"));
            }
            let projects: Value =
                serde_json::from_slice(&fs::read(app.root.join("src/lib/content/projects.json"))?)?;
            if !projects
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p["slug"] == *slug && p["link"] == "")
            {
                return Err(anyhow!("Unknown project slug"));
            }
            Ok(app.root.join("src/lib/content/devlogs.json"))
        }
    }
}
fn list_entries(app: &App, kind: &Kind) -> Result<Vec<Value>> {
    let data: Value = serde_json::from_slice(&fs::read(content_path(app, kind)?)?)?;
    Ok(match kind {
        Kind::Note => data.as_array().cloned().unwrap_or_default(),
        Kind::Devlog(slug) => data[slug].as_array().cloned().unwrap_or_default(),
    })
}

fn project_list(app: &App) -> Result<String> {
    let projects: Value =
        serde_json::from_slice(&fs::read(app.root.join("src/lib/content/projects.json"))?)?;
    let lines = projects
        .as_array()
        .ok_or_else(|| anyhow!("Invalid projects data"))?
        .iter()
        .filter(|project| project["link"] == "")
        .map(|project| {
            format!(
                "{} — slug: {}",
                project["title"].as_str().unwrap_or("?"),
                project["slug"].as_str().unwrap_or("?")
            )
        })
        .collect::<Vec<_>>();
    Ok(format!(
        "Project slugs for devlogs:\n{}\n\nExample: /new devlog calibur",
        lines.join("\n")
    ))
}

fn entry_list(app: &App, kind: &Kind) -> Result<String> {
    let entries = list_entries(app, kind)?;
    let (heading, example) = match kind {
        Kind::Note => ("Notes".to_string(), "/show note ID".to_string()),
        Kind::Devlog(slug) => (
            format!("Devlogs for {slug}"),
            format!("/show devlog {slug} ID"),
        ),
    };
    if entries.is_empty() {
        return Ok(format!("{heading}: no entries yet."));
    }
    let lines = entries
        .iter()
        .map(|entry| {
            format!(
                "{} ({})\nID: {}",
                entry["title"].as_str().unwrap_or("?"),
                entry["date"].as_str().unwrap_or("?"),
                entry["id"].as_str().unwrap_or("?")
            )
        })
        .collect::<Vec<_>>();
    Ok(format!(
        "{heading}:\n\n{}\n\nTo read one, replace ID in {example} with the ID shown above.",
        lines.join("\n\n")
    ))
}

fn format_entry(kind: &Kind, entry: &Value) -> String {
    let id = entry["id"].as_str().unwrap_or("?");
    let title = entry["title"].as_str().unwrap_or("?");
    let date = entry["date"].as_str().unwrap_or("?");
    let mut lines = vec![format!("{title} ({date})"), format!("ID: {id}")];
    if let Kind::Note = kind {
        lines.push(format!("Tags: {}", entry["tags"].as_str().unwrap_or("")));
        lines.push(format!(
            "Description: {}",
            entry["description"].as_str().unwrap_or("")
        ));
    }
    lines.push("\nContent:".to_string());
    if let Some(blocks) = entry["blocks"].as_array() {
        for block in blocks {
            match block["type"].as_str() {
                Some("text") => lines.push(block["text"].as_str().unwrap_or("").into()),
                Some("image") => lines.push(format!(
                    "Photo: {} ({})",
                    block["alt"].as_str().unwrap_or(""),
                    block["src"].as_str().unwrap_or("")
                )),
                _ => {}
            }
        }
    } else {
        if let Some(text) = entry["text"].as_str() {
            lines.push(text.into());
        }
        if entry["image"].is_object() {
            lines.push(format!(
                "Photo: {} ({})",
                entry["image"]["alt"].as_str().unwrap_or(""),
                entry["image"]["src"].as_str().unwrap_or("")
            ));
        }
    }
    let prefix = match kind {
        Kind::Note => format!("/edit note {id}"),
        Kind::Devlog(slug) => format!("/edit devlog {slug} {id}"),
    };
    lines.push(format!(
        "\nTo edit: {prefix} title (or date, tags, description, body where available)."
    ));
    lines.join("\n")
}
fn draft_value(draft: &Draft) -> Result<Value> {
    let mut value =
        json!({ "id": draft.id, "title": draft.title, "date": draft.date, "blocks": draft.blocks });
    if matches!(draft.kind, Kind::Note) {
        value["tags"] = json!(draft.tags);
        value["description"] = json!(draft.description);
    }
    Ok(value)
}
async fn mutate_entries(
    app: &App,
    kind: &Kind,
    f: impl FnOnce(&mut Vec<Value>) -> Result<()>,
) -> Result<()> {
    let path = content_path(app, kind)?;
    mutate_file(app, &path, |data| {
        let entries = match kind {
            Kind::Note => data
                .as_array_mut()
                .ok_or_else(|| anyhow!("Invalid notes data"))?,
            Kind::Devlog(slug) => data[slug]
                .as_array_mut()
                .ok_or_else(|| anyhow!("Invalid devlog data"))?,
        };
        f(entries)
    })
    .await
}
async fn mutate_file(
    app: &App,
    path: &Path,
    f: impl FnOnce(&mut Value) -> Result<()>,
) -> Result<()> {
    let _lock = app.publish_lock.lock().await;
    let old = fs::read(path)?;
    let mut data: Value = serde_json::from_slice(&old)?;
    f(&mut data)?;
    let temp = path.with_extension("json.tmp");
    fs::write(&temp, serde_json::to_vec_pretty(&data)?)?;
    fs::rename(temp, path)?;
    if let Err(error) = run_publish(app).await {
        fs::write(path, old)?;
        return Err(error.context("Content reverted after failed publish"));
    }
    Ok(())
}
async fn publish(app: &App) -> Result<()> {
    let _lock = app.publish_lock.lock().await;
    run_publish(app).await
}
async fn run_publish(app: &App) -> Result<()> {
    let status = tokio::process::Command::new("bash")
        .arg("scripts/publish.sh")
        .current_dir(&app.root)
        .status()
        .await?;
    if !status.success() {
        return Err(anyhow!("publish script exited with {status}"));
    }
    Ok(())
}

async fn send_chunks(bot: &Bot, chat: ChatId, text: String) -> Result<()> {
    let mut chunk = String::new();
    for character in text.chars() {
        if chunk.len() + character.len_utf8() > 3500 {
            bot.send_message(chat, &chunk).await?;
            chunk.clear();
        }
        chunk.push(character);
    }
    if !chunk.is_empty() {
        bot.send_message(chat, chunk).await?;
    }
    Ok(())
}

fn cleanup_draft(app: &App, draft: &Draft) {
    for block in &draft.blocks {
        if let Block::Image { src, .. } = block {
            if let Some(name) = src.strip_prefix("/uploads/") {
                if !name.contains('/') {
                    let _ = fs::remove_file(app.root.join("static/uploads").join(name));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_photo_text_keeps_submission_order() {
        let draft = Draft {
            kind: Kind::Note,
            id: "entry-test".into(),
            title: "A note".into(),
            date: "2026-10-17".into(),
            tags: "Robotics".into(),
            description: String::new(),
            blocks: vec![
                Block::Text {
                    text: "before".into(),
                },
                Block::Image {
                    src: "/uploads/photo.jpg".into(),
                    alt: "test".into(),
                },
                Block::Text {
                    text: "after".into(),
                },
            ],
            step: 5,
            editing: false,
        };
        let entry = draft_value(&draft).unwrap();
        assert_eq!(entry["blocks"][0]["text"], "before");
        assert_eq!(entry["blocks"][1]["src"], "/uploads/photo.jpg");
        assert_eq!(entry["blocks"][2]["text"], "after");
    }

    #[test]
    fn edit_targets_the_requested_project_and_field() {
        let (kind, id, field) =
            entry_args(&["/edit", "devlog", "calibur", "entry-7", "body"], true).unwrap();
        assert!(matches!(kind, Kind::Devlog(slug) if slug == "calibur"));
        assert_eq!(id, "entry-7");
        assert_eq!(field, "body");
    }

    #[test]
    fn show_puts_the_id_before_long_content() {
        let entry = json!({ "id": "legacy-note-1", "title": "Example", "date": "2026-10-17", "tags": "", "description": "", "text": "Body" });
        let shown = format_entry(&Kind::Note, &entry);
        assert!(shown.contains("ID: legacy-note-1\n"));
        assert!(shown.find("ID: legacy-note-1").unwrap() < shown.find("Body").unwrap());
    }
}
