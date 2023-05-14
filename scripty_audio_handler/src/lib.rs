#[macro_use]
extern crate tracing;

mod audio_handler;
mod connect;
mod consts;
mod disconnect;
mod error;
mod events;
mod types;

pub use audio_handler::AudioHandler;
pub use connect::connect_to_vc;
pub use disconnect::disconnect_from_vc;
pub use error::Error;
pub use scripty_audio::{check_model_language, get_model_languages};
use serenity::client::Context;
use songbird::{driver::DecodeMode, id::GuildId, Config};
pub use songbird::{error::JoinError, serenity::SerenityInit};

pub fn get_songbird() -> Config {
	Config::default().decode_mode(DecodeMode::Decode)
}

pub async fn check_voice_state(ctx: &Context, guild_id: GuildId) -> bool {
	songbird::get(ctx)
		.await
		.expect("failed to get songbird object")
		.get(guild_id)
		.map_or(false, |_| true)
}
