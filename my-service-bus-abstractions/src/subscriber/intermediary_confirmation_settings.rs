use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

/// If the handling of the batch takes longer than that - we start sending IntermediaryConfirm packets.
///
/// It has to be noticeably less than delivery_timeout on the broker side (30 sec by default),
/// since the broker kicks the whole connection off if the subscriber stays in OnDelivery state
/// longer than that.
const DEFAULT_INTERVAL_MICROS: u64 = 3_000_000;

static INTERVAL_MICROS: AtomicU64 = AtomicU64::new(DEFAULT_INTERVAL_MICROS);

/// Sets the IntermediaryConfirm interval for the readers which are created after this call.
/// Default value is 3 seconds
pub fn set_default_intermediary_confirmation_interval(value: Duration) {
    INTERVAL_MICROS.store(value.as_micros() as u64, Ordering::SeqCst);
}

pub fn get_default_intermediary_confirmation_interval() -> Duration {
    Duration::from_micros(INTERVAL_MICROS.load(Ordering::SeqCst))
}
