use std::sync::atomic::AtomicU64;
use std::time::Instant;

pub struct Stats {
    #[allow(unused)]
    pub started_at: Instant,
    pub total_commands: AtomicU64,
}

pub struct Ctx {
    pub stats: Stats,
}

impl Ctx {
    pub fn new() -> Self {
        Self {
            stats: Stats {
                started_at: Instant::now(),
                total_commands: AtomicU64::new(0),
            },
        }
    }
}
