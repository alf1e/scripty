use poise::CreateReply;
use scripty_bot_utils::checks::is_guild;
use serenity::{
	all::ButtonStyle,
	builder::{CreateActionRow, CreateButton},
	collector::ComponentInteractionCollector,
};

use crate::{Context, Error};

/// View and agree to Scripty's Terms of Service and Privacy Policy.
#[poise::command(
	prefix_command,
	slash_command,
	check = "is_guild",
	required_permissions = "MANAGE_GUILD"
)]
pub async fn terms_of_service(ctx: Context<'_>) -> Result<(), Error> {
	let resolved_language =
		scripty_i18n::get_resolved_language(ctx.author().id.0, ctx.guild_id().map(|g| g.0)).await;
	let db = scripty_db::get_db();

	let guild_id = ctx.guild_id().ok_or_else(Error::expected_guild)?;

	// as far as i can tell, there's not a nice way of doing this without two queries
	sqlx::query!(
		"INSERT INTO guilds (guild_id) VALUES ($1) ON CONFLICT ON CONSTRAINT guilds_pkey DO NOTHING",
		guild_id.get() as i64
	)
	.execute(db)
	.await?;
	let res = sqlx::query!(
		"SELECT agreed_tos FROM guilds WHERE guild_id = $1",
		guild_id.get() as i64
	)
	.fetch_one(db)
	.await?;

	if res.agreed_tos {
		ctx.say(format_message!(resolved_language, "already-agreed-to-tos"))
			.await?;
	} else {
		// send a message with the terms of service and privacy policy
		let m = ctx
			.send(
				CreateReply::new()
					.content(format_message!(resolved_language, "agreeing-to-tos"))
					.components(vec![CreateActionRow::Buttons(vec![
						CreateButton::new("tos_agree")
							.emoji('✅')
							.label("Agree")
							.style(ButtonStyle::Success),
						CreateButton::new("tos_disagree")
							.emoji('❎')
							.label("Disagree")
							.style(ButtonStyle::Danger),
					])]),
			)
			.await?;

		let maybe_interaction = ComponentInteractionCollector::new(&ctx.discord().shard)
			.timeout(std::time::Duration::from_secs(60))
			.author_id(ctx.author().id)
			.message_id(m.message().await?.id)
			.custom_ids(vec!["tos_agree".to_string(), "tos_disagree".to_string()])
			.await;

		if let Some(interaction) = maybe_interaction {
			if interaction.data.custom_id == "tos_agree" {
				sqlx::query!(
					"UPDATE guilds SET agreed_tos = true WHERE guild_id = $1",
					guild_id.get() as i64
				)
				.execute(db)
				.await?;
				m.edit(
					ctx,
					CreateReply::new()
						.content(format_message!(resolved_language, "tos-agree-success")),
				)
				.await?;
			} else {
				m.edit(
					ctx,
					CreateReply::new()
						.content(format_message!(resolved_language, "disagreed-to-tos")),
				)
				.await?;
			}
		} else {
			m.edit(
				ctx,
				CreateReply::new()
					.content(format_message!(resolved_language, "tos-agree-timed-out")),
			)
			.await?;
		}
	}

	Ok(())
}