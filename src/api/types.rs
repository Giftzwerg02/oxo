use serde::Serialize;

/// THIS STRUCT DOES NOT CONTAIN INFORMATION ABOUT THE ALREADY PLAYED TIME
/// WE WILL MAKE USE OF A SEPERATE ENDPOINT/STRUCT WITH SOCKETS TO HAVE A LIVE-UPDATE OF EACH TRACK
/// HAVE A LOOK AT `TrackUpdate`
#[derive(Serialize)]
pub struct Track {
    pub title: Option<String>,
    pub author: Author,
    pub thumbnail: Option<String>,
    pub length_secs: Option<u64>,
    pub url: Option<String>,
}

#[derive(Serialize)]
pub struct Author {
    pub name: Option<String>,
    pub icon_url: Option<String>,
}

#[derive(Serialize)]
pub struct TrackUpdate {
    pub position_secs: u64,
}
