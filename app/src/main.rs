use std::{
    error::Error,
    fmt::Display,
    path::PathBuf,
    result::Result::Ok,
    str::FromStr,
    thread,
    time::Duration,
};

use anyhow::{
    Context,
    Result,
};
use chrono::{
    DateTime,
    Local,
    NaiveDateTime,
    TimeZone,
};
use clap::Parser;
use lb_importer_services::{
    load_listens,
    ImportData::{
        ListenBrainz,
        Spotify,
    },
    ListenData,
};
use listenbrainz::raw::{
    request::{
        ListenType,
        Payload,
        SubmitListens,
    },
    Client,
};
use uuid::Uuid;


/// Import play history from spotify data dump into ListenBrainz
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// ListenBrainz API token
    #[arg(short, long, env = "LISTENBRAINZ_TOKEN")]
    token: Uuid,

    /// Url of the listenbrainz compatible API to import into
    #[arg(short, long)]
    url: Option<String>,

    /// Only import tracks played before this date/time
    #[arg(short, long, value_parser = parse_optional_tz)]
    before: Option<DateTime<Local>>,

    /// Only import tracks played after this date/time
    #[arg(short, long, value_parser = parse_optional_tz)]
    after: Option<DateTime<Local>>,

    /// Minimum play time in seconds in order to import
    #[arg(long, default_value_t = 30)]
    min_play_time: u16,

    /// How many listens to import per request
    #[arg(long, default_value_t = 1000)]
    batch_size: usize,

    /// One or more json files containing play history
    ///
    /// Expects file names to match the following patterns based on service source:
    ///
    ///     Spotify: endsong_\d+ | StreamingHistory\d+
    ///
    ///     ListenBrainz: \w+_lb-\d{4}-\d{2}-\d{2}
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

fn parse_optional_tz(dt: &str) -> Result<DateTime<Local>, impl Error> {
    dt.parse()
        .or_else(|_| NaiveDateTime::from_str(dt).map(|ndt| Local.from_local_datetime(&ndt).single().unwrap()))
}

fn print_err<E: Display>(e: E) {
    eprintln!("{e:#}");
}

fn main() -> Result<()> {
    let args = Args::parse_from(wild::args_os());

    #[cfg(debug_assertions)]
    dbg!(&args);

    let client = args.url.map(Client::new_with_url).unwrap_or_else(Client::new);

    let token = args.token.as_hyphenated().to_string();
    if !client.validate_token(token.as_str())?.valid {
        return Err(listenbrainz::Error::InvalidToken.into());
    }

    macro_rules! payload {
        ($it:expr) => {
            $it.filter(|ld| args.before.map(|dt| ld.listened_at() < dt.timestamp()).unwrap_or(true))
                .filter(|ld| args.after.map(|dt| dt.timestamp() < ld.listened_at()).unwrap_or(true))
                .map(Payload::from)
                .collect::<Vec<_>>()
        };
    }

    let min_play_ms = args.min_play_time as u32 * 1000;
    let mut tracks = args
        .files
        .iter()
        .filter_map(|p| load_listens(p).with_context(|| p.display().to_string()).map_err(print_err).ok())
        .flat_map(|v| match v {
            Spotify(sv) => payload!(sv.into_iter().filter(|h| h.ms_played >= min_play_ms)),
            ListenBrainz(lv) => payload!(lv.into_iter()),
        })
        .peekable();


    let mut total = 0usize;
    let mut batch = Vec::with_capacity(args.batch_size);
    while tracks.peek().is_some() {
        batch.extend(tracks.by_ref().take(args.batch_size));
        let resp = client
            .submit_listens(token.as_str(), SubmitListens {
                listen_type: ListenType::Import,
                payload: &batch,
            })
            .with_context(|| format!("Batch {total}-{}", total + batch.len()));

        #[cfg(debug_assertions)]
        dbg!(&resp);

        match resp {
            Err(e) => print_err(e),
            Ok(resp) => {
                total += batch.len();
                println!("Imported {} listens | {total} total", batch.len());

                if let Some(limit) = resp.rate_limit {
                    if limit.remaining == 0 {
                        println!("API rate limit reached; Will continue in {} seconds...", limit.reset_in);
                        thread::sleep(Duration::from_secs(limit.reset_in));
                    }
                }
            },
        }

        batch.clear();
    }

    anyhow::Ok(())
}
