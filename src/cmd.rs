use crate::{ctx::Ctx, store::Store, types::RespValue};

pub fn dispatch(args: &[RespValue], store: &Store, ctx: &Ctx) -> RespValue {
    // 1. contar
    ctx.stats
        .total_commands
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    // 2. sacar nombre
    if args.is_empty() {
        return RespValue::Error("ERR empty array".into());
    }
    let Some(cmd) = args[0].as_command() else {
        return RespValue::Error("ERR command must be a bulk string".into());
    };

    // 3. delegar
    match cmd.as_str() {
        "PING" => cmd_ping(args),
        "GET" => cmd_get(args, store),
        "SET" => cmd_set(args, store),
        "DEL" => cmd_del(args, store),
        "ECHO" => cmd_echo(args),
        "EXISTS" => cmd_exists(args, store),
        "DBSIZE" => cmd_dbsize(args, store),
        _ => RespValue::Error(format!("ERR unknown command: {cmd}")),
    }
}

fn cmd_dbsize(args: &[RespValue], store: &Store) -> RespValue {
    if args.len() >= 2 {
        return RespValue::Error("ERR wrong number of arguments for 'dbsize' command".into());
    }

    RespValue::Integer(store.len() as i64)
}

fn cmd_exists(args: &[RespValue], store: &Store) -> RespValue {
    if args.len() < 2 {
        return RespValue::Error("ERR wrong number of arguments for 'exists' command".into());
    }
    let mut found: i64 = 0;
    for arg in &args[1..] {
        let Some(key) = arg.as_key() else { continue };
        if store.exists(&key) {
            found += 1;
        }
    }
    RespValue::Integer(found)
}

fn cmd_del(args: &[RespValue], store: &Store) -> RespValue {
    if args.len() < 2 {
        return RespValue::Error("ERR wrong number of arguments for 'del' command".into());
    }
    let mut deleted: i64 = 0;
    for arg in &args[1..] {
        let Some(key) = arg.as_key() else { continue };
        if store.del(&key) {
            deleted += 1;
        }
    }
    RespValue::Integer(deleted)
}

fn cmd_echo(args: &[RespValue]) -> RespValue {
    if args.len() != 2 {
        return RespValue::Error("ERR wrong number of arguments for 'echo' command".into());
    }
    match &args[1] {
        RespValue::BulkString(b) => RespValue::BulkString(b.clone()),
        RespValue::SimpleString(s) => RespValue::BulkString(s.as_bytes().to_vec()),
        _ => RespValue::Error("ERR wrong type for 'echo'".into()),
    }
}

fn cmd_set(args: &[RespValue], store: &Store) -> RespValue {
    if args.len() != 3 {
        return RespValue::Error("ERR wrong number of arguments for 'set' command".into());
    }
    let Some(key_name) = args[1].as_key() else {
        return RespValue::Error("ERR wrong type for 'set' key".into());
    };
    if let RespValue::BulkString(value_bytes) = &args[2] {
        store.set(key_name, value_bytes.clone());
        RespValue::SimpleString("OK".to_string())
    } else {
        RespValue::Error("ERR wrong type for 'set' value".to_string())
    }
}

fn cmd_ping(_args: &[RespValue]) -> RespValue {
    RespValue::SimpleString("PONG".into())
}

fn cmd_get(args: &[RespValue], store: &Store) -> RespValue {
    if args.len() != 2 {
        return RespValue::Error("ERR wrong number of arguments for 'get' command".into());
    }
    let Some(key) = args[1].as_key() else {
        return RespValue::Error("ERR wrong type for 'get' key".into());
    };
    match store.get(&key) {
        Some(v) => RespValue::BulkString((*v).clone()),
        None => RespValue::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bulk(s: &str) -> RespValue {
        RespValue::BulkString(s.as_bytes().to_vec())
    }

    fn setup() -> (Store, Ctx) {
        (Store::new(), Ctx::new())
    }

    #[test]
    fn test_ping_and_lowercase() {
        let (store, ctx) = setup();
        assert_eq!(
            dispatch(&[bulk("PING")], &store, &ctx),
            RespValue::SimpleString("PONG".into())
        );
        // as_command() normaliza a uppercase
        assert_eq!(
            dispatch(&[bulk("ping")], &store, &ctx),
            RespValue::SimpleString("PONG".into())
        );
    }

    #[test]
    fn test_set_then_get() {
        let (store, ctx) = setup();
        assert_eq!(
            dispatch(&[bulk("SET"), bulk("k"), bulk("v")], &store, &ctx),
            RespValue::SimpleString("OK".into())
        );
        assert_eq!(
            dispatch(&[bulk("GET"), bulk("k")], &store, &ctx),
            RespValue::BulkString(b"v".to_vec())
        );
    }

    #[test]
    fn test_get_missing_returns_null() {
        let (store, ctx) = setup();
        assert_eq!(
            dispatch(&[bulk("GET"), bulk("nokey")], &store, &ctx),
            RespValue::Null
        );
    }

    #[test]
    fn test_get_arity_and_wrong_type() {
        let (store, ctx) = setup();
        assert!(matches!(
            dispatch(&[bulk("GET")], &store, &ctx),
            RespValue::Error(_)
        ));
        assert!(matches!(
            dispatch(&[bulk("GET"), bulk("a"), bulk("b")], &store, &ctx),
            RespValue::Error(_)
        ));
        // clave Integer no es BulkString -> error, no lookup de ""
        assert_eq!(
            dispatch(&[bulk("GET"), RespValue::Integer(123)], &store, &ctx),
            RespValue::Error("ERR wrong type for 'get' key".into())
        );
    }

    #[test]
    fn test_set_arity_and_wrong_type() {
        let (store, ctx) = setup();
        assert_eq!(
            dispatch(&[bulk("SET"), bulk("onlykey")], &store, &ctx),
            RespValue::Error("ERR wrong number of arguments for 'set' command".into())
        );
        assert_eq!(
            dispatch(
                &[bulk("SET"), RespValue::Integer(1), bulk("v")],
                &store,
                &ctx
            ),
            RespValue::Error("ERR wrong type for 'set' key".into())
        );
        assert!(matches!(
            dispatch(
                &[bulk("SET"), bulk("k"), RespValue::Integer(1)],
                &store,
                &ctx
            ),
            RespValue::Error(_)
        ));
    }

    #[test]
    fn test_echo() {
        let (store, ctx) = setup();
        assert_eq!(
            dispatch(&[bulk("ECHO"), bulk("hi")], &store, &ctx),
            RespValue::BulkString(b"hi".to_vec())
        );
        assert!(matches!(
            dispatch(&[bulk("ECHO")], &store, &ctx),
            RespValue::Error(_)
        ));
        assert!(matches!(
            dispatch(&[bulk("ECHO"), RespValue::Integer(1)], &store, &ctx),
            RespValue::Error(_)
        ));
    }

    #[test]
    fn test_del_multi_and_missing() {
        let (store, ctx) = setup();
        dispatch(&[bulk("SET"), bulk("a"), bulk("1")], &store, &ctx);
        dispatch(&[bulk("SET"), bulk("b"), bulk("2")], &store, &ctx);
        assert_eq!(
            dispatch(
                &[bulk("DEL"), bulk("a"), bulk("b"), bulk("nokey")],
                &store,
                &ctx
            ),
            RespValue::Integer(2)
        );
        // ya borradas -> 0
        assert_eq!(
            dispatch(&[bulk("DEL"), bulk("a")], &store, &ctx),
            RespValue::Integer(0)
        );
        // sin claves -> error de aridad (antes era panic/ok segun version)
        assert!(matches!(
            dispatch(&[bulk("DEL")], &store, &ctx),
            RespValue::Error(_)
        ));
    }

    #[test]
    fn test_exists_single_and_multi() {
        let (store, ctx) = setup();
        assert_eq!(
            dispatch(&[bulk("EXISTS"), bulk("k")], &store, &ctx),
            RespValue::Integer(0)
        );
        dispatch(&[bulk("SET"), bulk("k"), bulk("v")], &store, &ctx);
        assert_eq!(
            dispatch(&[bulk("EXISTS"), bulk("k")], &store, &ctx),
            RespValue::Integer(1)
        );
        assert_eq!(
            dispatch(&[bulk("EXISTS"), bulk("k"), bulk("nokey")], &store, &ctx),
            RespValue::Integer(1)
        );
        // sin claves -> error, no panic
        assert!(matches!(
            dispatch(&[bulk("EXISTS")], &store, &ctx),
            RespValue::Error(_)
        ));
    }

    #[test]
    fn test_dbsize() {
        let (store, ctx) = setup();
        assert_eq!(
            dispatch(&[bulk("DBSIZE")], &store, &ctx),
            RespValue::Integer(0)
        );
        dispatch(&[bulk("SET"), bulk("a"), bulk("1")], &store, &ctx);
        dispatch(&[bulk("SET"), bulk("b"), bulk("2")], &store, &ctx);
        assert_eq!(
            dispatch(&[bulk("DBSIZE")], &store, &ctx),
            RespValue::Integer(2)
        );
        dispatch(&[bulk("DEL"), bulk("a")], &store, &ctx);
        assert_eq!(
            dispatch(&[bulk("DBSIZE")], &store, &ctx),
            RespValue::Integer(1)
        );
        assert!(matches!(
            dispatch(&[bulk("DBSIZE"), bulk("extra")], &store, &ctx),
            RespValue::Error(_)
        ));
    }

    #[test]
    fn test_unknown_and_empty() {
        let (store, ctx) = setup();
        assert_eq!(
            dispatch(&[bulk("FOO")], &store, &ctx),
            RespValue::Error("ERR unknown command: FOO".into())
        );
        assert!(matches!(dispatch(&[], &store, &ctx), RespValue::Error(_)));
    }

    #[test]
    fn test_non_bulk_command_rejected() {
        let (store, ctx) = setup();
        assert_eq!(
            dispatch(&[RespValue::Integer(1)], &store, &ctx),
            RespValue::Error("ERR command must be a bulk string".into())
        );
        assert_eq!(
            dispatch(&[RespValue::SimpleString("PING".into())], &store, &ctx),
            RespValue::Error("ERR command must be a bulk string".into())
        );
    }
}
