# rudist
**redis** but in **rust** so is **rudist**, ahem. 

---

A tiny Redis-like server written in Rust.

`rudist` implements a small subset of Redis using the RESP wire protocol (the protocol used by `redis-cli`). It’s meant as a learning/systems project: networking, protocol parsing, concurrency, and a thread-safe in-memory store.

> Status: **experimental** — not production-ready.

## Features

- RESP parsing for:
  - Simple Strings (`+OK\r\n`)
  - Errors (`-ERR ...\r\n`)
  - Integers (`:123\r\n`)
  - Bulk Strings (`$<len>\r\n...\r\n`)
  - Arrays (`*<n>\r\n...`)
  - Null array (`*-1\r\n`)
- TCP server listening on `127.0.0.1:6379`
- In-memory key/value store with `RwLock<HashMap<...>>`
- Basic commands:
  - `PING` → `PONG`
  - `ECHO <msg>` → `<msg>`
  - `GET <key>` → bulk string or null
  - `SET <key> <value>` → `OK`
  - `DEL <key> [key ...]` → number of keys removed
  - `EXISTS <key> [key ...]` → number of keys found
  - `DBSIZE` → number of keys

## Requirements

- Rust toolchain (edition **2024**).

## Build & run

```bash
cargo build
cargo run
```

You should see something like:

```text
r > Listening in 127.0.0.1:6379
```

## Try it

### Using `redis-cli`

If you have Redis installed, you can use `redis-cli` to talk to `rudist`:

```bash
redis-cli -h 127.0.0.1 -p 6379 PING
redis-cli -h 127.0.0.1 -p 6379 SET mykey hello
redis-cli -h 127.0.0.1 -p 6379 GET mykey
redis-cli -h 127.0.0.1 -p 6379 DEL mykey
redis-cli -h 127.0.0.1 -p 6379 DBSIZE
```

### Using netcat (`nc`)

You can also send raw RESP frames:

```bash
# PING
printf '*1\r\n$4\r\nPING\r\n' | nc 127.0.0.1 6379

# SET mykey hello
printf '*3\r\n$3\r\nSET\r\n$5\r\nmykey\r\n$5\r\nhello\r\n' | nc 127.0.0.1 6379

# GET mykey
printf '*2\r\n$3\r\nGET\r\n$5\r\nmykey\r\n' | nc 127.0.0.1 6379
```

## Run tests

```bash
cargo test
```

## Roadmap / ideas

If you want to keep evolving this project, these are good next steps:

- Implement RESP Errors/Integers fully and improve invalid-frame handling
- Add observability: `INFO`, `MONITOR`, `SLOWLOG`
- Add key expiration (TTL)
- Add persistence (AOF) and/or snapshots
- Add async I/O (Tokio) instead of thread-per-connection
- Add CI (GitHub Actions): `fmt`, `clippy`, `test`

## License

Add a license file if you plan to share/use this beyond learning.
