# rudist — TODO

Acercar el código a lo que dice el portfolio
(`MONITOR`/`INFO`/`SLOWLOG`, reloj simulado, dump JSON).

Ahora mismo: `PING/GET/SET/ECHO/DEL/EXISTS/DBSIZE`, thread-per-connection
en `127.0.0.1:6379`, store en `RwLock<HashMap>`.

---

## Fase 1 — Dispatch fuera de `main.rs` (cerrada)

- [x] `src/cmd.rs` con `dispatch()` y `src/ctx.rs` con `Stats`/`Ctx` (compartidos con `Arc`)
- [x] `PING` -> `PONG`, `ECHO` devuelve bulk
- [x] `DEL` / `EXISTS` multi-key, devuelven a cuántas claves afectan
- [x] `DBSIZE` (con `Store::len`)
- [x] Comandos case-insensitive; comando que no es bulk -> `-ERR`
- [x] Errores de aridad como los de Redis (`wrong number of arguments for '...'`)
- [x] Tests llamando a `dispatch()` directamente
- [x] `cargo test` + `clippy` + probado con `redis-cli`

## Fase 2 — Observabilidad (lo que diferencia a rudist)

- [ ] `src/stats.rs`: `uptime`, `total_commands`, claves, clientes
- [ ] `INFO` (secciones `server`/`stats`/`keyspace` mínimas)
- [ ] `MONITOR`: broadcast de cada comando; el cliente en modo monitor solo escucha
- [ ] `SLOWLOG GET/LEN/RESET`: últimas lentas, umbral 10 ms configurable
- [ ] `DEBUG DUMP` / `RUDIST.DUMP`: el store en JSON
- [ ] Probar con `redis-cli`: `INFO`, y `MONITOR` en una terminal + comandos en otra

## Fase 3 — Reloj simulado + expiry

- [ ] `src/clock.rs`: `trait Clock` con `RealClock` y `ManualClock`
- [ ] Valores del store con expiración opcional
- [ ] `SET key val EX 10 / PX 5000`, `TTL`, `PTTL`
- [ ] Expiración lazy en `get()` + barrido cada 100 ms en un hilo aparte
- [ ] Tests con `ManualClock::advance(...)`, sin `sleep`
- [ ] `cargo test` sin flakiness

## Fase 4 — Después (opcional, no ahora)

- [ ] `INCR`/`DECR`
- [ ] `KEYS` / `SCAN` básico
- [ ] Persistencia AOF o snapshot
- [ ] Tokio en vez de thread-per-connection
- [ ] CI (GitHub Actions): `fmt`, `clippy`, `test`

## Reglas

- Cada fase = un commit que compile y pase tests.
- No abrir Fase 3 sin cerrar Fase 1.
- No tocar `VoltVault`, `mao`, `cstm` mientras tanto.
