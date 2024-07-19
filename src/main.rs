use log::{info, error};
use eyre::eyre;
use serde::Deserialize;
use serenity::{
    async_trait,
    client::ClientBuilder,
    http::Http,
    model::{
        channel::{Embed, Message},
        prelude::{ChannelId, MessageType},
        webhook::Webhook,
    },
    prelude::*,
};

use std::collections::HashMap;
use std::fs::File;

struct WebhookMap;

impl TypeMapKey for WebhookMap {
    type Value = HashMap<ChannelId, Webhook>;
}

async fn handle_message(
    webhook: &Webhook,
    ctx: &Context,
    new_message: Message,
) -> eyre::Result<()> {
    let Message {
        referenced_message,
        content,
        author,
        embeds,
        attachments,
        ..
    } = new_message;

    let mut embeds = embeds
        .into_iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()?;

    let mut files = Vec::new();

    for attachment in attachments {
        let content_type = attachment
            .content_type
            .as_ref()
            .ok_or_else(|| eyre!("no content type"))?;

        if content_type.starts_with("image") {
            embeds.push(Embed::fake(|e| {
                e.title("image").image(attachment.url).colour((47, 49, 54))
            }));
        } else {
            files.push(attachment.url);
        }
    }

    for sticker_item in new_message.sticker_items {
        let sticker_url = sticker_item
            .image_url()
            .ok_or_else(|| eyre!("could not get sticker url"))?;

        embeds.push(Embed::fake(|e| {
            e.title(format!("`{}` sticker", sticker_item.name))
                .image(sticker_url)
                .colour((47, 49, 54))
        }))
    }

    if !files.is_empty() {
        embeds.push(Embed::fake(|e| {
            e.title("attachments")
                .description(files.join("\n"))
                .colour((47, 49, 54))
        }));
    }

    let formatted_username = {
        let mut username = author.name.clone();

        if author.bot {
            username += " [BOT]";
        }

        if new_message.kind == MessageType::InlineReply {
            let topic = referenced_message.ok_or_else(|| eyre!("referenced message not found"))?;

            username += " (in reply to ";
            username += &topic.author.name;
            username += ")"
        }

        username
    };

    webhook
        .execute(&ctx.http, false, |w| {
            w.content(content)
                .avatar_url(author.face())
                .username(formatted_username)
                .embeds(embeds)
        })
        .await?;

    Ok(())
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let data = ctx.data.read().await;
        let map = data.get::<WebhookMap>().unwrap();

        let Some(webhook) = map.get(&msg.channel_id) else {
            return;
        };

        let id = msg.id;
        let author = msg.author.name.clone();
        let channel = msg.channel_id;

        info!("plagiarizing message #{id} in channel #{channel} from {author}");

        if let Err(why) = handle_message(webhook, &ctx, msg).await {
            let error_message = format!("failed to plagiarize message #{id} from {author} ({why})");

            error!("{error_message}");

            webhook
                .execute(&ctx.http, false, |w| w.content(&error_message))
                .await
                .ok();
        }
    }
}

#[derive(Debug, Deserialize)]
struct Config {
    token: String,
    channels: HashMap<String, String>
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv::dotenv()?;
    color_eyre::install()?;

    pretty_env_logger::init_timed();

    let config: Config = serde_json::from_reader(File::open("./config.json")?)?;

    let http = Http::new(&config.token);

    let mut webhooks = HashMap::new();

    for (id, url) in config.channels {
        let channel_id = ChannelId(id.parse()?);
        let webhook = Webhook::from_url(&http, &url).await?;

        webhooks.insert(channel_id, webhook);
    }

    let mut client = ClientBuilder::new_with_http(http, GatewayIntents::all())
        .type_map_insert::<WebhookMap>(webhooks)
        .event_handler(Handler)
        .await?;

    info!("starting client...");

    if let Err(why) = client.start().await {
        error!("client error: {why}");
    }

    Ok(())
}
