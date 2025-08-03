use crate::error::AppError;
use serde_json::Value;
use ureq::Agent;

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
        let data: Value =
            serde_json::from_reader(reader).map_err(|_| AppError::InternalServerError)?;

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

        // 1. From recording
        let mut genre = Self::extract_genre_from_tags(&recording["tags"]);
        println!("le genre: {:?}", genre);

        // 2. From release
        if genre.is_none() {
            println!("on teste release");
            if let Some(release_id) = recording["releases"].get(0).and_then(|r| r["id"].as_str()) {
                let release_url = format!(
                    "https://musicbrainz.org/ws/2/release/{}?fmt=json&inc=tags",
                    release_id
                );
                let release_res = agent
                    .get(&release_url)
                    .set("User-Agent", "Swaptun/1.0 (contact@swaptun.local)")
                    .call()
                    .map_err(|_| AppError::InternalServerError)?;
                let release_data: Value = serde_json::from_reader(release_res.into_reader())
                    .map_err(|_| AppError::InternalServerError)?;

                genre = Self::extract_genre_from_tags(&release_data["tags"]);
            }
            println!("le genre: {:?}", genre);
        }

        // 3. From artist
        if genre.is_none() {
            println!("on teste artist");
            if let Some(artist_id) = recording["artist-credit"][0]["artist"]["id"].as_str() {
                let artist_url = format!(
                    "https://musicbrainz.org/ws/2/artist/{}?fmt=json&inc=tags",
                    artist_id
                );
                let artist_res = agent
                    .get(&artist_url)
                    .set("User-Agent", "Swaptun/1.0 (contact@swaptun.local)")
                    .call()
                    .map_err(|_| AppError::InternalServerError)?;
                let artist_data: Value = serde_json::from_reader(artist_res.into_reader())
                    .map_err(|_| AppError::InternalServerError)?;

                genre = Self::extract_genre_from_tags(&artist_data["tags"]);
            }
            println!("genre :{:?}", genre);
        }

        Ok(TrackInfo {
            title,
            artist,
            musicbrainz_id: id,
            release_date,
            genre,
        })
    }

    fn extract_genre_from_tags(tags: &Value) -> Option<String> {
        tags.as_array()?
            .iter()
            .filter_map(|tag| tag.get("name").and_then(|n| n.as_str()))
            .next()
            .map(|s| s.to_string())
    }
}

pub async fn get_track_metadata(title: &str, artist: &str) -> Result<Option<TrackInfo>, AppError> {
    let service = MusicBrainzService::new();
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    match service.search_track(title, artist) {
        Ok(track_info) => Ok(Some(track_info)),
        Err(AppError::InternalServerError) => Ok(None), // Pas trouvé
        Err(e) => Err(e),
    }
}
