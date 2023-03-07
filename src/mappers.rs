use songbird::tracks::TrackHandle;

use crate::api::types::{Author, Track};

impl From<TrackHandle> for Track {
    fn from(track: TrackHandle) -> Self {
        let metadata = track.metadata().clone();
        Self {
            title: metadata.title,
            thumbnail: metadata.thumbnail,
            url: metadata.source_url,
            author: Author {
                name: metadata.artist,
                // TODO: refactor this string from embed_ext.rs to be globally available
                icon_url: Some("https://raw.githubusercontent.com/Giftzwerg02/oxo/19bdb259f38a0fde3231e9957019b889e5d3280c/resources/music.png".into())
            },
            length_secs: metadata.duration.map(|d| d.as_secs())
        }
    }
}