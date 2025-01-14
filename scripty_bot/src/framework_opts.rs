use poise::{FrameworkOptions, PrefixFrameworkOptions};
use scripty_bot_utils::error::handler::on_error;
use scripty_commands::build_commands;
use serenity::{builder::CreateAllowedMentions, model::id::UserId, prelude::GatewayIntents};

pub fn get_framework_opts() -> FrameworkOptions<scripty_bot_utils::Data, scripty_bot_utils::Error> {
	FrameworkOptions {
		commands: build_commands(),
		on_error: |error| Box::pin(on_error(error)),
		command_check: Some(scripty_bot_utils::entity_block::check_block),
		pre_command: scripty_bot_utils::handler::pre_command,
		post_command: scripty_bot_utils::handler::post_command,
		// Only support direct user pings by default
		allowed_mentions: Some(
			CreateAllowedMentions::default()
				.empty_roles()
				.empty_users()
				.replied_user(true),
		),
		prefix_options: PrefixFrameworkOptions {
			prefix: Some("~".to_string()),
			execute_self_messages: false,
			execute_untracked_edits: true,
			mention_as_prefix: true,
			..Default::default()
		},
		owners: scripty_config::get_config()
			.owners
			.iter()
			.map(|id| UserId::new(*id))
			.collect(),

		..Default::default()
	}
}

pub fn get_gateway_intents() -> GatewayIntents {
	GatewayIntents::GUILDS
		| GatewayIntents::GUILD_MEMBERS
		| GatewayIntents::GUILD_WEBHOOKS
		| GatewayIntents::GUILD_VOICE_STATES
		| GatewayIntents::GUILD_MESSAGES
		| GatewayIntents::DIRECT_MESSAGES
		| GatewayIntents::MESSAGE_CONTENT
}
