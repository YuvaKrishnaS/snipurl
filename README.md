# SnipURL

A fast, self-hosted URL shortener with per-link click analytics. Built with Axum, SQLite, and sqlx.

## Features

- Shorten any URL with a random or custom code
- Redirect tracking with user-agent logging per click
- Per-link stats: click count and last 20 click logs
- Global analytics: total links, total clicks, top 5 links by clicks
- Web UI at `/` for creating links (no install needed)
- API key auth on all write and admin endpoints
- Custom JSON 404 responses via fallback handler
- `DATABASE_URL` env var for persistent storage on Railway volumes

## Routes

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/` | No | Web UI with shortener form and live stats |
| GET | `/health` | No | Health check |
| POST | `/shorten` | Yes | Create a short link |
| GET | `/links` | Yes | All links sorted by click count |
| GET | `/analytics` | Yes | Global totals and top 5 most-clicked links |
| GET | `/stats/:code` | Yes | Per-link data and last 20 click logs |
| DELETE | `/links/:code` | Yes | Delete a link and its click history |
| GET | `/:code` | No | Redirect to the original URL |

## Running locally

```
API_KEY=secret cargo run
```

Optional env vars:

| Variable | Default | Purpose |
|----------|---------|---------|
| `API_KEY` | required | Auth header value for protected endpoints |
| `BASE_URL` | `http://localhost:3000` | Prefix used in short URL responses |
| `DATABASE_URL` | `sqlite:snipurl.db?mode=rwc` | SQLite connection string |
| `PORT` | `3000` | Port to bind on |

## API examples

```bash
# shorten a URL
curl -X POST http://localhost:3000/shorten \
  -H "Content-Type: application/json" \
  -H "x-api-key: secret" \
  -d '{"url": "https://example.com/very/long/path"}'

# custom code
curl -X POST http://localhost:3000/shorten \
  -H "Content-Type: application/json" \
  -H "x-api-key: secret" \
  -d '{"url": "https://example.com", "custom_code": "home"}'

# list all links
curl -H "x-api-key: secret" http://localhost:3000/links

# global analytics
curl -H "x-api-key: secret" http://localhost:3000/analytics

# per-link stats
curl -H "x-api-key: secret" http://localhost:3000/stats/home

# delete a link
curl -X DELETE -H "x-api-key: secret" http://localhost:3000/links/home
```

## Deploying to Railway

1. Push your project to a GitHub repo
2. Create a new project on [Railway](https://railway.app) and connect the repo
3. Add a Volume and mount it at `/data`
4. Set environment variables:
   - `API_KEY` - your secret key
   - `BASE_URL` - your Railway public URL (e.g. `https://snipurl.up.railway.app`)
   - `DATABASE_URL` - `sqlite:/data/snipurl.db?mode=rwc`
5. Railway detects the `Dockerfile` and deploys automatically

