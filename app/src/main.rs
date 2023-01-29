#![feature(result_option_inspect)]

use std::{
    fmt::Display,
    fs::File,
    io::BufReader,
    thread,
    time::Duration,
};

use anyhow::{
    Context,
    Result,
};
use clap::Parser;
use lb_importer_services::{
    load_listenbrainz,
    load_spotify,
    service::spotify::Listen,
    ListenData,
};
use listenbrainz::raw::{
    request::{
        ListenType,
        Payload,
        StrType,
        SubmitListens,
    },
    Client,
};
use time::{
    format_description::well_known::Rfc3339,
    OffsetDateTime,
};

use crate::args::{
    Args,
    Service::{
        ListenBrainz,
        Spotify,
    },
    SpotifyArgs,
};

mod args;


fn print_err(e: &impl Display) {
    eprintln!("{e:#}");
}

fn main() -> Result<()> {
    let args = Args::parse_from(wild::args_os());

    #[cfg(debug_assertions)]
    dbg!(&args);

    let client = args.url.map_or_else(Client::new, Client::new_with_url);

    let token = args.token.as_hyphenated().to_string();
    if !client.validate_token(token.as_str())?.valid {
        return Err(listenbrainz::Error::InvalidToken.into());
    }

    let files = args.files.iter().filter_map(|p| {
        File::open(p)
            .map(BufReader::new)
            .inspect(|_| println!("Importing file '{}'", p.display()))
            .with_context(|| p.display().to_string())
            .inspect_err(print_err)
            .ok()
    });

    macro_rules! filtered {
        ($load:expr) => {
            files
                .filter_map(|f| $load(f).inspect_err(print_err).ok())
                .flatten()
                .filter(|ld| args.before.map(|dt| ld.listened_at() < dt.unix_timestamp()).unwrap_or(true))
                .filter(|ld| args.after.map(|dt| dt.unix_timestamp() < ld.listened_at()).unwrap_or(true))
        };
    }
    macro_rules! submit {
        ($it:expr) => {
            submit($it, args.batch_size, &client, &token);
        };
    }
    match args.service {
        ListenBrainz => {
            submit!(filtered!(load_listenbrainz));
        },
        Spotify(SpotifyArgs { min_play_time }) => {
            let mut listens: Vec<_> = filtered!(load_spotify)
                .filter(|l| l.ms_played >= u32::from(min_play_time * 1000))
                .collect();
            listens.sort_unstable_by_key(ListenData::listened_at);

            let listens = dedup_spotify(&listens, u64::from(min_play_time));

            submit!(listens.into_iter());
        },
    }

    anyhow::Ok(())
}

fn dedup_spotify(listens: &Vec<Listen>, time_threshold: u64) -> Vec<&Listen> {
    // const ALL_REASONS: [&str; 13] = ["appload","backbtn","clickrow","endplay","fwdbtn","logout","playbtn","remote","trackdone","trackerror","unexpected-exit","unexpected-exit-while-paused","unknown"];
    const SKIP_REASONS: [&str; 4] = ["logout", "remote", "trackerror", "unknown"];
    fn is_skip_reason(re: &Option<String>) -> bool {
        re.as_ref()
            .map_or(false, |re| re.starts_with("unexpected-") || SKIP_REASONS.iter().any(|&sr| re == sr))
    }

    listens.iter().rev().fold(Vec::with_capacity(listens.len()), |mut acc, l| {
        if let Some(&prev) = acc.last() {
            if prev.spotify_track_uri != l.spotify_track_uri
                || !(is_skip_reason(&l.reason_end) || prev.listened_at().abs_diff(l.listened_at()) <= time_threshold)
            {
                acc.push(l);
            } else {
                eprintln!("Ignoring duplicate listen for `{}` by `{}`", l.track_name(), l.artist_name());
                #[cfg(debug_assertions)]
                dbg!(l);
            }
        } else {
            acc.push(l);
        }
        acc
    })
}


fn submit<T, A, R>(listens: impl Iterator<Item = impl Into<Payload<T, A, R>>>, batch_size: usize, client: &Client, token: &str)
where
    T: StrType,
    A: StrType,
    R: StrType,
{
    #[rustfmt::skip]
    #[derive(Default)]
    struct Counts { total: usize, success: usize, fail: usize }

    let mut counts = Counts::default();
    let mut batch = Vec::with_capacity(batch_size);
    let mut listens = listens.map(Into::into).peekable();
    while listens.peek().is_some() {
        batch.extend(listens.by_ref().take(batch_size));
        let resp = client
            .submit_listens(token, SubmitListens {
                listen_type: ListenType::Import,
                payload: &batch,
            })
            .with_context(|| format!("Batch {}-{}", counts.total, counts.total + batch.len()));
        counts.total += batch.len();

        #[cfg(debug_assertions)]
        dbg!(&resp);

        match resp {
            Err(e) => {
                counts.fail += batch.len();
                print_err(&e);

                macro_rules! dt {
                    ($p:expr) => {
                        $p.and_then(|p| p.listened_at)
                            .and_then(|ts| OffsetDateTime::from_unix_timestamp(ts).ok())
                            .and_then(|dt| dt.format(&Rfc3339).ok())
                            .expect("Payload should always have valid listened_at")
                    };
                }
                print_err(&format!("> Rerun batch using: --after {} --before {}", dt!(batch.last()), dt!(batch.first())));
            },
            Ok(resp) => {
                counts.success += batch.len();
                println!("Imported {} listens | Succeeded: {}, Failed: {}, Total: {}", batch.len(), counts.success, counts.fail, counts.total);

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
}
