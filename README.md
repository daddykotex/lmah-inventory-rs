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


## Notes

### Deployment

Experimented with Fly's Sprites.

Use a Nix Flake to try:

```
nix run github:jamiebrynes7/sprite-cli-nix
```

But transfering the file is harder than it seems.
