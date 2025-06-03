use ureq::Agent;
use serde_json::Value;
use crate::error::AppError;

pub struct MusicBrainzService;

#[derive(Debug, serde::Serialize)]
pub struct TrackInfo {
    pub title: String,
    pub artist: String,
    pub musicbrainz_id: String,
    pub release_date: Option<String>,
    pub genre: Option<String>,
}


impl MusicBrainzService {
    pub fn new() -> Self {
        MusicBrainzService
    }

    pub fn search_track(&self, title: &str, artist: &str) -> Result<TrackInfo, AppError> {
        let agent = Agent::new();
        let query = format!("recording:\"{}\" AND artist:\"{}\"", title, artist);
        let url = format!(
            "https://musicbrainz.org/ws/2/recording/?query={}&fmt=json&limit=1&inc=artist-credits+releases+tags",
            query
        );

        let res = agent
            .get(&url)
            .set("User-Agent", "Swaptun/1.0 (contact@swaptun.local)")
            .call()
            .map_err(|_| AppError::InternalServerError)?;

        let reader = res.into_reader();
        let data: Value = serde_json::from_reader(reader).map_err(|_| AppError::InternalServerError)?;


        let recording = &data["recordings"][0];

        let title = recording["title"].as_str().unwrap_or_default().to_string();
        let id = recording["id"].as_str().unwrap_or_default().to_string();
        let artist = recording["artist-credit"][0]["artist"]["name"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        let release_date = recording["first-release-date"]
            .as_str()
            .map(|s| s.to_string());

        let genre = recording["tags"]
            .get(0)
            .and_then(|tag| tag["name"].as_str())
            .map(|s| s.to_string());

        Ok(TrackInfo {
            title,
            artist,
            musicbrainz_id: id,
            release_date,
            genre,
        })
    }
}
