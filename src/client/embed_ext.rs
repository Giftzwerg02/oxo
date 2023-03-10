use std::{fmt::Display, time::Duration};

use poise::serenity_prelude::{Colour, CreateEmbed, Timestamp};
use songbird::{input::Metadata, tracks::TrackState};

pub trait CreateEmbedExt {
    const ERROR_COLOUR: Colour = Colour::DARK_RED;
    const WARN_COLOUR: Colour = Colour::ORANGE;
    const NORMAL_COLOUR: Colour = Colour::BLITZ_BLUE;
    const UNSAFE_FERRIS: &'static str = "https://raw.githubusercontent.com/Giftzwerg02/oxo/33856f5c3ad1549de092f7f58a83b05e1b060398/resources/unsafe-ferris-transparent.png";
    const FERRIS: &'static str = "https://github.com/Giftzwerg02/oxo/blob/d7b3db148524841058dcf8399e849b79ba7c6bf6/resources/ferris.png";
    const COULD_NOT_LOAD_FILE: &'static str = "https://raw.githubusercontent.com/Giftzwerg02/oxo/19bdb259f38a0fde3231e9957019b889e5d3280c/resources/could_not_load_file.png";
    const MUSIC_ICON: &'static str = "https://raw.githubusercontent.com/Giftzwerg02/oxo/19bdb259f38a0fde3231e9957019b889e5d3280c/resources/music.png";

    fn error_styling(&mut self) -> &mut Self;
    fn normal_styling(&mut self) -> &mut Self;
    fn warn_styling(&mut self) -> &mut Self;

    fn now(&mut self) -> &mut Self;

    fn oxo_footer(&mut self) -> &mut Self;

    fn song_embed(&mut self, song_metadata: &Metadata, track_state: &TrackState) -> &mut Self;
    fn info_embed(&mut self, msg: impl Display) -> &mut Self;
}

impl CreateEmbedExt for CreateEmbed {
    fn error_styling(&mut self) -> &mut Self {
        self.colour(Self::ERROR_COLOUR)
            .thumbnail(Self::UNSAFE_FERRIS)
            .oxo_footer()
            .now()
    }

    fn now(&mut self) -> &mut Self {
        self.timestamp(Timestamp::now())
    }

    fn oxo_footer(&mut self) -> &mut Self {
        self.footer(|f| f.text("XOXO"))
    }

    fn normal_styling(&mut self) -> &mut Self {
        self.colour(Self::NORMAL_COLOUR)
            .thumbnail(Self::FERRIS)
            .oxo_footer()
            .now()
    }

    fn info_embed(&mut self, msg: impl Display) -> &mut Self {
        self.normal_styling().title("Oki doki!").description(msg)
    }

    fn warn_styling(&mut self) -> &mut Self {
        self.colour(Self::WARN_COLOUR)
            .thumbnail(Self::UNSAFE_FERRIS)
            .oxo_footer()
            .now()
    }

    fn song_embed(&mut self, song_metadata: &Metadata, track_state: &TrackState) -> &mut Self {
        let title = song_metadata.title.clone().unwrap_or_else(|| "N/A".into());
        let author = format!(
            "By: {}",
            song_metadata.artist.clone().unwrap_or_else(|| "N/A".into())
        );

        let thumbnail = song_metadata
            .thumbnail
            .clone()
            .unwrap_or_else(|| Self::COULD_NOT_LOAD_FILE.into());

        let total_duration = song_metadata.duration.unwrap_or_default();
        let already_played = track_state.position;

        fn duration_format(duration: &Duration) -> String {
            let seconds = duration.as_secs();
            let minutes = seconds / 60;
            let hours = minutes / 60;
            let minutes = minutes % 60;
            let seconds = seconds % 60;

            if hours > 0 {
                format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
            } else {
                format!("{:02}:{:02}", minutes, seconds)
            }
        }

        let description = format!(
            "Duration: `{} / {}`",
            duration_format(&already_played),
            duration_format(&total_duration)
        );

        let url = song_metadata
            .source_url
            .clone()
            .unwrap_or_else(|| "https://www.youtube.com/watch?v=dQw4w9WgXcQ".into());

        self.normal_styling()
            .title(title)
            .author(|a| a.name(author).icon_url(Self::MUSIC_ICON))
            .thumbnail(thumbnail)
            .description(description)
            .url(url)
    }
}
