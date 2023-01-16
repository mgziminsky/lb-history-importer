use std::path::PathBuf;

use anyhow::Result;
use clap::{
    arg,
    ArgGroup,
    Command,
    Parser,
};
use time::{
    format_description::{
        well_known::Rfc3339,
        FormatItem,
    },
    macros::format_description,
    Date,
    OffsetDateTime,
    PrimitiveDateTime,
    UtcOffset,
};
use uuid::Uuid;

/// Import play history from a history dump into a `ListenBrainz` instance
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// ListenBrainz API token
    #[arg(short, long, env = "LISTENBRAINZ_TOKEN")]
    pub token: Uuid,

    /// Url of the listenbrainz compatible API to import into
    #[arg(short, long)]
    pub url: Option<String>,

    /// Only import tracks played before this date/time
    #[arg(short, long, value_parser = parse_datetime)]
    pub before: Option<OffsetDateTime>,

    /// Only import tracks played after this date/time
    #[arg(short, long, value_parser = parse_datetime)]
    pub after: Option<OffsetDateTime>,

    /// How many listens to import per request
    #[arg(long, default_value_t = 1000)]
    pub batch_size: usize,

    /// The service where the dump came from
    #[command(flatten)]
    pub service: Service,

    /// One or more json files containing play history
    #[arg(required = true)]
    pub files: Vec<PathBuf>,
}

#[derive(Debug)]
pub(crate) enum Service {
    Spotify(SpotifyArgs),
    ListenBrainz,
}

impl clap::Args for Service {
    fn augment_args(cmd: clap::Command) -> clap::Command {
        const HEADING: &str = "Services";
        cmd.group(ArgGroup::new("service").args(["spotify", "listenbrainz"]).required(true))
            .arg(
                arg!(--spotify)
                    .help_heading(HEADING)
                    .help("Import files from a spotify dump")
                    .long_help(r"endsong_\d+.json | StreamingHistory\d+.json"),
            )
            .arg(
                arg!(--listenbrainz)
                    .help_heading(HEADING)
                    .help("Import files from a listenbrainz dump")
                    .long_help(r"\w+_lb-\d{4}-\d{2}-\d{2}.json"),
            )
            .args(
                SpotifyArgs::augment_args(Command::new(""))
                    .get_arguments()
                    .cloned()
                    .map(|sa| sa.requires("spotify").help_heading("Spotify Options")),
            )
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command { Self::augment_args(cmd) }

    fn group_id() -> Option<clap::Id> { Some("service".into()) }
}

impl clap::FromArgMatches for Service {
    fn from_arg_matches(matches: &clap::ArgMatches) -> std::result::Result<Self, clap::Error> {
        if matches.get_flag("spotify") {
            Ok(Service::Spotify(SpotifyArgs::from_arg_matches(matches)?))
        } else if matches.get_flag("listenbrainz") {
            Ok(Self::ListenBrainz)
        } else {
            Err(clap::Error::new(clap::error::ErrorKind::MissingRequiredArgument))
        }
    }

    fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> std::result::Result<(), clap::Error> {
        match self {
            Service::Spotify(ref mut a) => a.update_from_arg_matches(matches),
            Service::ListenBrainz => Ok(()),
        }
    }
}

#[derive(clap::Args, Debug)]
pub(crate) struct SpotifyArgs {
    /// Minimum play time in seconds for a track to be imported
    #[arg(long, default_value_t = 30)]
    pub min_play_time: u16,
}


fn parse_datetime(dt: &str) -> Result<OffsetDateTime> {
    const FMTS_DT: &[&[FormatItem]] = &[
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
        format_description!("[year]-[month]-[day] [hour]:[minute]"),
        format_description!("[year]-[month]-[day] [hour]"),
    ];
    const FMTS_DATE: &[&[FormatItem]] = &[format_description!("[year]-[month]-[day]")];

    let local_tz = UtcOffset::current_local_offset()?;
    OffsetDateTime::parse(dt, &Rfc3339).or_else(|e| {
        FMTS_DT
            .iter()
            .find_map(|fmt| PrimitiveDateTime::parse(dt, fmt).ok())
            .or_else(|| {
                FMTS_DATE
                    .iter()
                    .find_map(|fmt| Date::parse(dt, fmt).ok())
                    .and_then(|d| d.with_hms(0, 0, 0).ok())
            })
            .map(|pdt| pdt.assume_offset(local_tz))
            .ok_or(e.into())
    })
}
