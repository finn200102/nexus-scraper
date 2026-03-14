# Nexus Scraper

A web novel and fanfiction scraper written in Rust. Downloads stories from multiple hosting platforms and exports them as JSON or HTML.

## Features

- Scrape stories from multiple fiction hosting sites
- Fetch individual chapters, chapter lists, or full story content
- Retrieve author information and their published stories
- Filter stories by series, rating, word count, and time range
- Python bindings for use as a library
- Command-line interface for quick downloads

## Supported Sites

| Site | Identifier | Notes |
|------|------------|-------|
| FanFiction.net | `fanfiction` | Stories, chapters, author pages |
| Archive of Our Own | `archive` | AO3 stories and chapters |
| SpaceBattles | `spacebattles` | Forum-hosted fiction |
| Royal Road | `royalroad` | Web novels and serials |
| Webnovel | `webnovel` | Chinese web novels, requires browser proxy |

## Requirements

- **Rust** (1.70+) - Install via [rustup](https://rustup.rs/)
- **Flaresolver** - Required for bot protection bypass

  Start Flaresolver on `http://localhost:8191/v1` before running commands:

  ```sh
  docker run -d -p 8191:8191 --name flaresolverr ghcr.io/flaresolverr/flaresolverr:latest
  ```

## Installation

```sh
# Clone the repository
git clone https://github.com/yourusername/nexus-scraper.git
cd nexus-scraper

# Build the CLI
cargo build -p nexus-cli

# Run directly
cargo run -p nexus-cli -- --help
```

## CLI Usage

```sh
cargo run -p nexus-cli -- [COMMAND] [OPTIONS]
```

### Commands

#### fetch-chapter

Download a single chapter as HTML.

```sh
cargo run -p nexus-cli -- fetch-chapter \
  --site fanfiction \
  --story-id 12345678 \
  --chapter-number 1
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--site` | string | required | Site identifier (e.g., `fanfiction`) |
| `--story-id` | u64 | `0` | Story ID from the site |
| `--chapter-id` | u64 | `0` | Chapter ID (site-dependent) |
| `--chapter-number` | u32 | `1` | Chapter number to fetch |

**Output:** `chapter{N}.html` - Raw HTML file

---

#### fetch-chapters

Fetch chapter metadata (titles, IDs, numbers) without content.

```sh
cargo run -p nexus-cli -- fetch-chapters \
  --site fanfiction \
  --story-id 12345678
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--site` | string | required | Site identifier |
| `--story-id` | u64 | required | Story ID |

**Output:** `chapters{story_id}.json`

---

#### fetch-chapters-content

Fetch all chapters with full text content.

```sh
cargo run -p nexus-cli -- fetch-chapters-content \
  --site fanfiction \
  --story-id 12345678
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--site` | string | required | Site identifier |
| `--story-id` | u64 | required | Story ID |

**Output:** `chapters{story_id}.json` - Full chapter content

---

#### fetch-author-stories

Get all stories by a specific author.

```sh
cargo run -p nexus-cli -- fetch-author-stories \
  --site fanfiction \
  --author-id 12345 \
  --author-name "AuthorName"
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--site` | string | required | Site identifier |
| `--author-id` | u64 | `0` | Author's user ID |
| `--author-name` | string | `"UNKNOWN"` | Author's username |

**Output:** `author_{author_id}_stories.json`

---

#### fetch-author

Get author information from a story.

```sh
cargo run -p nexus-cli -- fetch-author \
  --site fanfiction \
  --story-id 12345678
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--site` | string | required | Site identifier |
| `--story-id` | u64 | required | Story ID |

**Output:** `author_from_story{story_id}.json`

---

#### fetch-stories

Get stories from a site's listing page.

```sh
cargo run -p nexus-cli -- fetch-stories \
  --site fanfiction \
  --sortby-id 0 \
  --num-pages 2
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--site` | string | required | Site identifier |
| `--sortby-id` | u32 | `0` | Sort order (site-specific) |
| `--num-pages` | u32 | `2` | Number of pages to fetch |

**Output:** `stories_{sortby_id}.json`

---

#### fetch-stories-by-series

Filter stories by series/category with advanced filters.

```sh
cargo run -p nexus-cli -- fetch-stories-by-series \
  --site fanfiction \
  --medium-name "book" \
  --series-name "Harry-Potter" \
  --sortby-id 4 \
  --rating-id 10 \
  --word-count 40 \
  --time-range 0 \
  --num-pages 2
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--site` | string | required | Site identifier |
| `--medium-name` | string | required | Content type (e.g., `book`, `crossover`) |
| `--series-name` | string | required | Series/filter name (e.g., `Harry-Potter`, `popular`) |
| `--sortby-id` | u32 | `4` | Sort order |
| `--rating-id` | u32 | `10` | Rating filter |
| `--word-count` | u32 | `1` | Minimum word count |
| `--time-range` | u32 | `0` | Time range (days) |
| `--num-pages` | u32 | `1` | Number of pages to fetch |

**Output:** `stories_{series_name}.json`

**Returns:** Each story includes:
- `story_id`, `story_name`, `author_name`, `author_id`
- `description`, `img_url`
- `word_count`, `reviews`, `favorites`, `follows`
- `publish_date`, `updated_date`, `status`
- `chapter_count` (when available)

---

#### get-story-data-from-url

Parse a story URL to extract all metadata. The `--site` flag is optional for most sites - it will be auto-detected from the URL.

```sh
# With explicit site
cargo run -p nexus-cli -- get-story-data-from-url \
  --site fanfiction \
  --url "https://www.fanfiction.net/s/12345678/1/Story-Title"

# Auto-detect site from URL (recommended)
cargo run -p nexus-cli -- get-story-data-from-url \
  --url "https://www.webnovel.com/book/story-name_123456"
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--site` | string | auto-detect | Site identifier (optional) |
| `--url` | string | required | Full story URL |

**Output:** `story.json` - Complete story metadata

---

## Python Bindings

Use nexus-scraper as a Python library.

### Installation

```sh
# Build the Python package
cd pybindings
pip install maturin
maturin develop
```

### Usage

```python
from pybindings import PySite

# Create a site instance
site = PySite("fanfiction")

# Fetch story from URL
story = site.fetch_story_from_url(
    "https://www.fanfiction.net/s/12345678/1/Story-Title"
)

print(f"Title: {story.story_name}")
print(f"Author: {story.author_name}")
print(f"Chapters: {len(story.chapters)}")

# Fetch a specific chapter
chapter = site.fetch_chapter(
    story_id=12345678,
    chapter_id=0,
    chapter_number=1
)
print(f"Chapter title: {chapter.title}")
```

### PySite Methods

| Method | Parameters | Returns |
|--------|------------|---------|
| `fetch_story_from_url` | `url: str` | `PyStory` |
| `fetch_chapter` | `story_id`, `chapter_id`, `chapter_number` | `PyChapter` |

### Return Types

**PyStory:**
```python
{
    "site": str,
    "story_name": str,
    "story_id": int,
    "author_name": str,
    "author_id": int,
    "chapters": List[PyChapter],
    "description": str,
    "img_url": str
}
```

**PyChapter:**
```python
{
    "site": str,
    "title": str,
    "text": str,
    "chapter_number": int,
    "chapter_id": int
}
```

## Architecture

```
nexus-scraper/
├── nexus-core/          # Core library
│   ├── src/
│   │   ├── models.rs   # Data structures (Story, Chapter, Author)
│   │   ├── sites/      # Site implementations
│   │   │   ├── mod.rs  # Site trait definition
│   │   │   ├── fanfiction.rs
│   │   │   ├── archive.rs
│   │   │   ├── spacebattles.rs
│   │   │   ├── royalroad.rs
│   │   │   └── webnovel.rs
│   │   ├── parser/     # HTML parsing logic per site
│   │   ├── network.rs  # HTTP client, proxy utilities, date parsing
│   │   └── error.rs    # Error types
│   ├── tests/          # Integration tests
│   │   └── fixtures/   # HTML fixtures for testing
│   └── Cargo.toml
├── nexus-cli/           # Command-line interface
│   ├── src/main.rs     # CLI commands and handlers
│   └── Cargo.toml
├── pybindings/         # Python bindings
│   ├── src/lib.rs      # PyO3 bindings
│   └── Cargo.toml
└── Cargo.toml          # Workspace configuration
```

### Key Components

- **Site Trait** (`nexus-core/src/sites/mod.rs`): Defines the interface for all site implementations. Implement this trait to add support for new sites.

- **Models** (`nexus-core/src/models.rs`): Serde-serializable structures for stories, chapters, and authors.

- **Parsers** (`nexus-core/src/parser/`): HTML scraping and extraction logic per site.

## Testing

Run tests with cargo:

```sh
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p nexus-core

# Run specific test
cargo test -p nexus-core test_parse_date
```

### Test Structure

Tests are located in:
- **Unit tests**: Inline in source files (e.g., `nexus-core/src/network.rs`)
- **Integration tests**: `nexus-core/tests/`
- **Fixtures**: `nexus-core/tests/fixtures/` - Sample HTML for parsing tests

### Adding New Parser Tests

1. Create HTML fixture in `nexus-core/tests/fixtures/{site}/`
2. Add test in `nexus-core/tests/test_{site}.rs`
3. Use `include_str!("fixtures/{site}/file.html")` to load fixtures

## Troubleshooting

### "Flaresolver not responding"

Ensure Flaresolver is running:
```sh
docker ps | grep flaresolverr
# If not running:
docker start flaresolverr
```

### "Unknown site" error

Check the site identifier matches exactly: `fanfiction`, `archive`, `spacebattles`, `royalroad`, or `webnovel`.

### Rate limiting

Add delays between requests or reduce `--num-pages` to avoid IP blocking.

## License

MIT
