# Rocket Powered Pastebin (`rktpb` | [`paste.rs`])

A pastebin that does just enough to be _really_ useful.

  - [x] Really fast, really lightweight.
  - [x] Renders _markdown_ like GitHub.
  - [x] Highlights `source code`.
  - [x] Returns plain text, too.
  - [x] Has a simple API usable via CLI.
  - [x] Has support for CORS.
  - [x] Limits paste upload sizes.
  - [x] No database: uses the file system.
  - [x] Automatically deletes stale pastes.

This pastebin powers [`paste.rs`], a public instance. Further usage details can
be found there.

[`paste.rs`]: https://paste.rs

## Usage

**R**ocket **P**owered Paste**b**in (`rktpb`) is written in
[Rust](https://rust-lang.org) with [Rocket](https://rocket.rs). To start the
server, use `cargo`:

```sh
# clone the repository
git clone https://github.com/SergioBenitez/rktpb

# change into directory: the `static` folder needs to be in CWD before running
cd rktpb

# compile and start the server with the default config
cargo run
```

## Configuration

Configuration is provided via [environment variables](#environment-variables) or
a [TOML file](#toml-file). A set of defaults is always provided.

The complete list of configurable parameters is below:

| Name               | Default Value               | Description                               |
|--------------------|-----------------------------|-------------------------------------------|
| `id_length`        | `3`                         | paste ID length
| `upload_dir`       | `"upload"`                  | directory to save uploads in              |
| `paste_limit`      | `"384KiB"`                  | maximum paste upload file size            |
| `max_age`          | `"30 days"`                 | how long a paste is considered fresh      |
| `reap_interval`    | `"5 minutes"`               | how often to reap stale uploads           |
| `server_url`       | `"http://{address}:{port}"` | URL server is reachable at                |
|                    |                             |                                           |
| `cors.{host}`      | `["{HTTP method}"..]`       | allow CORS {HTTP methods} for {host}      |
|                    |                             |                                           |
| `address`          | `"127.0.0.1"`               | address to listen on                      |
| `port`             | `8000`                      | port to listen on                         |
| `keep_alive`       | `5`                         | HTTP keep-alive in seconds                |
| `ident`            | `"Rocket"`                  | server `Ident` header                     |
| `ip_header`        | `"X-Real-IP"`               | header to inspect for client IP           |
| `log_level`        | `"normal"`                  | console log level                         |
| `cli_colors`       | `true`                      | enable (detect TTY) or disable CLI colors |
|                    |                             |                                           |
| `shutdown.ctrlc`   | `true`                      | whether `<ctrl-c>` initiates a shutdown   |
| `shutdown.signals` | `["term", "hup"]`           | signals that initiate a shutdown          |
| `shutdown.grace`   | `5`                         | grace period length in seconds            |
| `shutdown.mercy`   | `5`                         | mercy period length in seconds            |
|                    |                             |                                           |
| `source_code_url`  | (this repo)                 | a link to your instance's source code     |

You'll definitely want to configure the values in the first two categories, from
`id_length` to `cors`.

You should likely use the defaults for the rest.

### Environment Variables

Use an environment variable name equivalent to the parameter name prefixed with
`PASTE_`:

```sh
PASTE_ID_LENGTH=10 PASTE_MAX_AGE="1 year" ./rktpb
```

To set structured data via environment variables, such as CORS, use [TOML-like
syntax](https://docs.rs/figment/latest/figment/providers/struct.Env.html):

```sh
PASTE_CORS='{"http://example.com"=["get", "post"]}' ./rktpb
```

### TOML File

See [`Paste.toml.template`](Paste.toml.template) for a template with all of the
defaults set as well as a dummy `cors` configuration for `http://example.com`
that allows the `options`, `get`, `post`, and `delete` HTTP methods.

```sh
mv Paste.toml.template Paste.toml
```

By default, the application searches for a file called `Paste.toml` in the CWD.
The path to the file can be overridden by setting `PASTE_CONFIG`. For example,
to use a file named `rktpb.toml`, use `PASTE_CONFIG="rktpb.toml" ./rktpb`.

## Deploying

To deploy, build in release mode and ship/run the resulting binary along with
`static/`, `templates/`, and any config:

```sh
# build in release mode for `${TARGET}`
cargo build --release --target ${TARGET}

# create a tarball of everything that's needed
tar -cvzf "rktpb.tar.gz" \
    Paste.toml static templates \
    -C target/${TARGET}/release rktpb
```

However you choose to deploy, you'll need to ensure that the CWD at the time the
server is started contains the `static` and `templates` directories as well as
the config file, if one is used.

Note that when the server is compiled in `release` mode, the `[release]` section
of a TOML config file can be used to override config values; the same is true
when compiled in `debug` mode with `[debug]`.

## License

Rocket Powered Pastebin (`rktpb` | [`paste.rs`])  
Copyright © 2020 Sergio Benitez

This program is free software: you can redistribute it and/or modify it under
the terms of the [GNU Affero General Public License version 3 (GNU AGPLv3) as
published by the Free Software
Foundation](https://www.gnu.org/licenses/agpl-3.0.en.html#license-text). This
program is distributed in the hope that it will be useful, but **WITHOUT ANY
WARRANTY**; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
A PARTICULAR PURPOSE. See the GNU AGPLv3 [LICENSE](LICENSE) for more details.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project shall be licensed under the GNU AGPLv3 License,
without any additional terms or conditions.
