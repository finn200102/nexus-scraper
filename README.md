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

- fetch a single chapter from fanfiction.net
```sh
cargo run -p nexus-cli -- fetch-chapter   --site fanfiction   --story-id ID<int64> --chapter-number Number<int32>
```

Some sites use a chapter number and some use a chapter id.

## Requirements

This applications needs Flaresolver to run on http://localhost:8191/v1 to work.

