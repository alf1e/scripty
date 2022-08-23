use crate::{Context, Error};
use poise::CreateReply;
use serenity::builder::CreateEmbed;

mod claim;
mod remove;

pub use claim::*;
pub use remove::*;

/// Premium commands
#[poise::command(prefix_command, slash_command)]
pub async fn premium(ctx: Context<'_>) -> Result<(), Error> {
    let resolved_language =
        scripty_i18n::get_resolved_language(ctx.author().id.0, ctx.guild_id().map(|g| g.0)).await;

    ctx.send(
        CreateReply::default().ephemeral(true).embed(
            CreateEmbed::default()
                .title(format_message!(
                    resolved_language,
                    "root-command-invoked-title"
                ))
                .description(format_message!(
                    resolved_language,
                    "root-command-invoked-description",
                    contextPrefix: ctx.prefix(),
                    commandName: "premium"
                )),
        ),
    )
    .await?;
    Ok(())
}