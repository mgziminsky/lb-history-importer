```
> lb-history-importer --help

Import listen data from dump files to a listenbrainz compatible service

Usage: lb-history-importer [OPTIONS] --token <TOKEN> <--spotify|--listenbrainz> <FILES>...

Arguments:
  <FILES>...
          One or more json files containing play history

Options:
  -t, --token <TOKEN>
          ListenBrainz API token

          [env: LISTENBRAINZ_TOKEN=]

  -u, --url <URL>
          Url of the listenbrainz compatible API to import into

  -b, --before <BEFORE>
          Only import tracks played before this date/time

  -a, --after <AFTER>
          Only import tracks played after this date/time

      --batch-size <BATCH_SIZE>
          How many listens to import per request

          [default: 1000]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Services:
      --spotify
          endsong_\d+.json | StreamingHistory\d+.json

      --listenbrainz
          \w+_lb-\d{4}-\d{2}-\d{2}.json

Spotify Options:
      --min-play-time <MIN_PLAY_TIME>
          Minimum play time in seconds for a track to be imported

          [default: 30]
```
