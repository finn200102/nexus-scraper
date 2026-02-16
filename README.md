# A webnovel downloader written in rust

## Commands

- help
```sh
cargo run -p nexus-cli -- --help
```

- command structure
```sh
cargo run -p nexus-cli -- <SUBCOMMAND> [OPTIONS]
```

- resolve story data from URL (auto-detects site)
```sh
cargo run -p nexus-cli -- get-story-data-from-url --url "https://www.fanfiction.net/s/14540056/1/Paradise-Found"
```

- optional explicit site override
```sh
cargo run -p nexus-cli -- get-story-data-from-url --site fanfiction --url "https://www.fanfiction.net/s/14540056/1/Paradise-Found"
```

- launch TUI scraper/reader
```sh
cargo run -p nexus-tui
```

- fetch a single chapter from fanfiction.net
```sh
cargo run -p nexus-cli -- fetch-chapter   --site fanfiction   --story-id ID<int64> --chapter-number Number<int32>
```

Some sites use a chapter number and some use a chapter id.

## Requirements

This applications needs Flaresolver to run on http://localhost:8191/v1 to work.

