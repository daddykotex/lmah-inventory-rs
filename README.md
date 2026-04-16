# LMAH Inventory rewrite in rust

Use rust to learn the language but also improve performance and the footprint over the initial implementation in Scala.

## Requirements

- devenv
- direnv

Direnv will load the environment through devenv. This will get you the right rust tool chain setup as well as load your `.env` (see [below](#environment-variables)).

## Environment variables

You need a .env before you can go through the rest of this README.
Copy the `.env.example` into `.env` and update has necessary.

## Database

The data is stored into a SQLite database.

We will use [`sqlx-cli`](https://crates.io/crates/sqlx-cli) to handle the database and its migrations. Install it with `cargo`:

```bash
cargo install sqlx-cli --no-default-features --features sqlite
```

Initialize with `sqlx database create"`.

Apply migrations with: `sqlx migrate run`.

### CLI migration

To migrate off Airtable, I've exported the data as JSON (from the other app), and I wrote the `cli` binary in this project to do so.

You can run it like:

```bash
# target is also loaded from the env var DATABASE_URL
cargo run --bin cli -- load --src data/db.json --target "sqlite://data/lmah.db"
```

You want to start from a clean database every time you run the import:

```bash
rm data/lmah.db && sqlx database create && sqlx migrate run
```

> Note: by default, `sqlx database create` uses `DATABASE_URL`

## Development

### Server

We use `watchexec` (installed through `devenv`) along with `systemfd` to have hot reload.

> Shamelessly borrowed from [blog post](https://lucumr.pocoo.org/2025/1/19/what-is-systemfd/)

```
cargo install systemfd
```

Run the server in development mode with:

```bash
systemfd --no-pid -s http::3000 -- watchexec -r -- cargo run --bin server -- --db-url "sqlite://data/lmah.db
```


## Notes

### Deployment

Experimented with Fly's Sprites.

Use a Nix Flake to try:

```
nix run github:jamiebrynes7/sprite-cli-nix
```

But transferring the file is harder than it seems.
