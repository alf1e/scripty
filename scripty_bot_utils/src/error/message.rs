use crate::{Context, Error};
use poise::CreateReply;
use serenity::builder::{
    CreateAttachment, CreateEmbed, CreateEmbedAuthor, CreateMessage, ExecuteWebhook,
};
use serenity::model::webhook::Webhook;

pub async fn send_err_msg(
    ctx: Context<'_>,
    title: impl Into<String>,
    description: impl Into<String>,
) {
    let embed = CreateEmbed::default()
        .title(title)
        .color((255, 0, 0))
        .description(description);

    let response = ctx.send(CreateReply::default().embed(embed.clone())).await;
    if let Err(e) = response {
        warn!("failed to send message while handling error: {}", e);
        let response = ctx
            .author()
            .direct_message(ctx.discord(), CreateMessage::default().embed(embed))
            .await;
        if let Err(e) = response {
            error!("failed to DM user while handling error: {}", e)
        }
    }
}

pub async fn log_error_message(
    ctx: &Context<'_>,
    mut err: Error,
    invocation_context: Option<String>,
) {
    // build embed
    let mut e = CreateEmbed::default();
    // build message
    let mut m = ExecuteWebhook::default();

    if let Some(inv_ctx) = invocation_context {
        e = e.title(format!("Error while {}", inv_ctx));
    } else {
        e = e.title("Error while doing something");
    }

    let fmt_bt = format!("{:#?}", err.backtrace());
    if fmt_bt.len() > 2048 {
        e = e.field("Backtrace", "See attached file", false);
        m = m.add_file(CreateAttachment::bytes(
            fmt_bt.into_bytes(),
            "backtrace.txt",
        ));
    } else {
        e = e.field("Backtrace", &fmt_bt, false);
    }

    e = e.field("Error (debug)", format!("{:?}", err), false);
    e = e.field("Error (display)", err.to_string(), false);

    // cache the cache
    let cache = ctx.discord().cache.clone();

    let (guild_id, guild_name) = if let Some(guild_id) = ctx.guild_id() {
        let guild_name = cache
            .guild(guild_id)
            .map_or_else(|| "unknown guild".to_string(), |g| g.name.clone());

        e = e.field("Guild ID", guild_id.to_string(), false);
        e = e.field("Guild Name", &guild_name, true);

        (Some(guild_id), Some(guild_name))
    } else {
        e = e.field("Guild ID", "None (DM ctx)", false);
        e = e.field("Guild Name", "None (DM ctx)", true);

        (None, None)
    };

    let channel_id = ctx.channel_id();
    e = e.field("Channel ID", channel_id.to_string(), false);

    let author = ctx.author();
    let author_id = author.id;
    let author_name = author.tag();
    let author_pfp = author.face();
    e = e.author(
        CreateEmbedAuthor::new(format!("{} ({})", author_name, author_id)).icon_url(author_pfp),
    );

    m = m.embed(e);

    let cfg = scripty_config::get_config();
    let dctx = ctx.discord();
    let hook = match Webhook::from_url(dctx, &cfg.error_webhook).await {
        Ok(hook) => hook,
        Err(e) => {
            error!("failed to fetch error webhook: {}", e);
            return;
        }
    };
    if let Err(e) = hook.execute(dctx, false, m).await {
        error!("failed to log error to discord: {}", e);
    }

    error!(?guild_id, ?guild_name, %channel_id, %author_id, %author_name, "error while doing something: {}", err);
}