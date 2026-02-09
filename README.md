# Social OSINT (Rust Edition)
A high-performance Rust implementation of the Social OSINT tool for gathering emails from Instagram, LinkedIn, and Twitter, with integrated PwnDB leak checking.

[![forthebadge](https://forthebadge.com/images/badges/made-with-rust.svg)](https://forthebadge.com)

## Features

- **Instagram**: Search users by hashtag, location, username; extract emails from profiles and bios
- **LinkedIn**: Search companies and people; retrieve contact information
- **Twitter**: Search tweets and extract emails (placeholder implementation)
- **PwnDB**: Check for leaked credentials via Tor proxy
- **Security**: Anti-ban features including User-Agent rotation, request jitter, and rate limiting
- **Performance**: Async/await with Tokio, connection pooling, and parallel processing
- **Observability**: Structured logging with tracing

## Installation

### Prerequisites

- Rust 1.70+ (`rustup` recommended)
- Tor service (for PwnDB functionality)

### Build

```bash
cargo build --release
```

The binary will be located at `target/release/socialosint`.

## Configuration

Create a `credentials.json` file with your Instagram and LinkedIn credentials:

```json
{
  "instagram": {
    "username": "your_instagram_username",
    "password": "your_instagram_password"
  },
  "linkedin": {
    "email": "your_linkedin_email",
    "password": "your_linkedin_password"
  }
}
```

## Usage

### Instagram

**Search by hashtag:**
```bash
./target/release/socialosint --credentials credentials.json --instagram --hashtag-ig security
```

**Search by location:**
```bash
# First, find location IDs
./target/release/socialosint --credentials credentials.json --instagram --info "New York"

# Then search by location ID
./target/release/socialosint --credentials credentials.json --instagram --location 12345
```

**Target specific user:**
```bash
./target/release/socialosint --credentials credentials.json --instagram --target-ig username
```

**Get followers/followings:**
```bash
./target/release/socialosint --credentials credentials.json --instagram --target-ig username --followers-ig
./target/release/socialosint --credentials credentials.json --instagram --target-ig username --followings-ig
```

### LinkedIn

**Search companies:**
```bash
./target/release/socialosint --credentials credentials.json --linkedin --search-companies "Google"
```

**Get company employees:**
```bash
./target/release/socialosint --credentials credentials.json --linkedin --search-companies "Google" --employees
```

**Search users:**
```bash
./target/release/socialosint --credentials credentials.json --linkedin --search-users-in "John Doe"
```

**Get user contact info:**
```bash
./target/release/socialosint --credentials credentials.json --linkedin --target-in username
```

### Twitter

**Search tweets by hashtag:**
```bash
./target/release/socialosint --credentials credentials.json --twitter --hashtag-tw security --limit 100
```

**Search user tweets:**
```bash
./target/release/socialosint --credentials credentials.json --twitter --target-tw username --all-tw
```

### PwnDB Integration

**Check for leaks:**
```bash
# Start Tor service first
brew services start tor  # macOS
# or
sudo systemctl start tor  # Linux

# Run with PwnDB
./target/release/socialosint --credentials credentials.json --instagram --target-ig username --pwndb
```

### Save Results

```bash
./target/release/socialosint --credentials credentials.json --instagram --hashtag-ig security --output results.txt
```

## CLI Arguments

### General
- `--credentials <FILE>` - JSON credentials file (required)
- `--output <FILE>` - Save results to file
- `--pwndb` - Enable PwnDB leak checking
- `--tor-proxy <PROXY>` - Tor proxy address (default: `127.0.0.1:9050`)

### Instagram
- `--instagram` - Enable Instagram mode
- `--info <QUERY>` - Get location IDs
- `--location <ID>` - Search by location
- `--hashtag-ig <TAG>` - Search by hashtag
- `--target-ig <USERNAME>` - Target specific user
- `--search-users-ig <QUERY>` - Search users
- `--my-followers` - Get own followers
- `--my-followings` - Get own followings
- `--followers-ig` - Get target's followers
- `--followings-ig` - Get target's followings

### LinkedIn
- `--linkedin` - Enable LinkedIn mode
- `--company <ID>` - Get company info
- `--search-companies <QUERY>` - Search companies
- `--employees` - Get employees
- `--my-contacts` - Get own contacts
- `--user-contacts <ID>` - Get user's contacts
- `--search-users-in <QUERY>` - Search users
- `--target-in <USERNAME>` - Target specific user
- `--add-contacts` - Send connection requests to all
- `--add-a-contact <ID>` - Send connection request to one user

### Twitter
- `--twitter` - Enable Twitter mode
- `--limit <N>` - Tweet limit (default: 100)
- `--year <YEAR>` - Filter by year
- `--since <DATE>` - Filter tweets since date
- `--until <DATE>` - Filter tweets until date
- `--profile-full` - Full profile scraping
- `--all-tw` - All tweets
- `--target-tw <USERNAME>` - Target user
- `--hashtag-tw <TAG>` - Search hashtag
- `--followers-tw` - Get followers
- `--followings-tw` - Get followings

## Security Features

- **User-Agent Rotation**: Random browser User-Agents
- **Request Jitter**: 500ms-2s delays between requests
- **Rate Limiting**: Per-domain throttling
- **Cloudflare Detection**: Automatic blocking detection
- **Cookie Management**: Persistent session handling
- **Connection Pooling**: Reuse HTTP connections

## Performance

- **Async/Await**: Tokio runtime for concurrent operations
- **Connection Pooling**: Reuse connections across requests
- **Parallel Processing**: Multiple targets processed concurrently
- **Memory Efficient**: Streaming results, minimal allocations

## Limitations

- **Twitter**: Requires Twitter API v2 bearer token for full functionality. Falls back to Nitter scraping if unavailable.
- **HaveIBeenPwned**: Requires HIBP API key for full functionality. Set `HIBP_API_KEY` environment variable.

## Environment Variables

- `TWITTER_BEARER_TOKEN`: (Optional) Twitter API v2 bearer token. Get one at https://developer.twitter.com/
- `HIBP_API_KEY`: (Optional) HaveIBeenPwned API key for breach checking. Get one at https://haveibeenpwned.com/API/Key
- `RUST_LOG`: Set logging level (e.g., `debug`, `info`, `warn`, `error`)

## Troubleshooting

**Instagram login fails:**
- Verify credentials in `credentials.json`
- Instagram may require 2FA or CAPTCHA solving

**LinkedIn authentication fails:**
- Check credentials
- LinkedIn may require email verification

**PwnDB not working:**
- Ensure Tor service is running: `brew services list` or `systemctl status tor`
- Verify proxy address: default is `127.0.0.1:9050`

**Rate limiting:**
- Increase delays in `src/http.rs`
- Use fewer concurrent requests

## Development:

**Run tests:**
```bash
cargo test
```

**Run with debug logging:**
```bash
RUST_LOG=debug ./target/release/socialosint --credentials credentials.json --instagram --target-ig username
```

**Build optimized:**
```bash
cargo build --release
```

## License
- [MIT](./LICENSE)

## Credits

Rust port by Krishna Pranav. Original Python implementation: [socialosint](https://github.com/krishpranav/socialosint).
