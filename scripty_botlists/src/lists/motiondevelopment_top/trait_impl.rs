use reqwest::{Client, Error as ReqwestError, RequestBuilder};

use crate::common::{PostStats, StatPoster};

pub struct MotionDevelopmentTop {
	token:  String,
	bot_id: u64,
}

impl MotionDevelopmentTop {
	pub fn new(token: String, bot_id: u64) -> Self {
		Self { token, bot_id }
	}

	pub fn token(&self) -> &str {
		&self.token
	}

	pub fn bot_id(&self) -> u64 {
		self.bot_id
	}
}

#[async_trait]
impl StatPoster for MotionDevelopmentTop {
	async fn post_stats(&self, client: Client, stats: PostStats) -> Result<bool, ReqwestError> {
		let request: RequestBuilder = client
			.post(format!(
				"https://motiondevelopment.top/api/v1.2/bots/{}/stats",
				self.bot_id
			))
			.header("Key", &self.token)
			.json(&super::models::PostStats {
				guilds: stats.server_count,
			});
		let response = request.send().await?.error_for_status()?;
		if response.status() != reqwest::StatusCode::OK {
			return Ok(false);
		}
		let body: super::models::PostStatsResponse = response.json().await?;
		Ok(body.success)
	}
}