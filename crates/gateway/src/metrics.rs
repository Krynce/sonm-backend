//! Counters for the gateway, published as one log line a minute.
//!
//! The gateway had no observability at all: no tracing, no metrics, and a
//! `sentry::capture_message` on *every* failed event, so one bad publisher or one mass
//! disconnect turned into a Sentry flood. These counters answer the questions we actually
//! ask — are clients connected, is Ready slow, are we dropping events — and the summary
//! carries the error counts to Sentry at most once per interval.
//
// ponytail: plain atomics and a log line, no Prometheus endpoint. Add one if we ever want
// history or alerting rather than "what is it doing right now".

use std::{
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
    time::Duration,
};

/// How often the summary is logged
const SUMMARY_INTERVAL: Duration = Duration::from_secs(60);

/// Currently connected WebSocket clients
pub static CONNECTIONS: AtomicUsize = AtomicUsize::new(0);

/// Pub/sub topics currently subscribed to, summed over all connections
pub static SUBSCRIPTIONS: AtomicUsize = AtomicUsize::new(0);

/// Connections accepted since the last summary
pub static ACCEPTED: AtomicU64 = AtomicU64::new(0);

/// Ready payloads sent since the last summary
pub static READY: AtomicU64 = AtomicU64::new(0);

/// Time spent building and sending those Ready payloads
pub static READY_MICROS: AtomicU64 = AtomicU64::new(0);

/// Events forwarded to clients since the last summary
pub static EVENTS_SENT: AtomicU64 = AtomicU64::new(0);

/// Events we received on a topic but could not deserialise
pub static DECODE_ERRORS: AtomicU64 = AtomicU64::new(0);

/// Failures writing to a client socket, excluding ordinary closes
pub static SEND_ERRORS: AtomicU64 = AtomicU64::new(0);

/// Failures reading from a client socket, excluding ordinary closes
pub static READ_ERRORS: AtomicU64 = AtomicU64::new(0);

/// Record one completed Ready payload
pub fn record_ready(elapsed: Duration) {
    READY.fetch_add(1, Ordering::Relaxed);
    READY_MICROS.fetch_add(elapsed.as_micros() as u64, Ordering::Relaxed);
}

/// Move the global subscription gauge by the change in one connection's set
///
/// `current` is that connection's last reported count and is updated in place; pass `0` when
/// the connection goes away.
pub fn track_subscriptions(current: &mut usize, now: usize) {
    if now > *current {
        SUBSCRIPTIONS.fetch_add(now - *current, Ordering::Relaxed);
    } else {
        SUBSCRIPTIONS.fetch_sub(*current - now, Ordering::Relaxed);
    }

    *current = now;
}

/// One interval's counters, read and reset together
#[derive(Default, Debug, PartialEq)]
struct Interval {
    accepted: u64,
    ready: u64,
    ready_micros: u64,
    events_sent: u64,
    decode_errors: u64,
    send_errors: u64,
    read_errors: u64,
}

impl Interval {
    /// Read every interval counter and reset it to zero
    fn take() -> Self {
        Self {
            accepted: ACCEPTED.swap(0, Ordering::Relaxed),
            ready: READY.swap(0, Ordering::Relaxed),
            ready_micros: READY_MICROS.swap(0, Ordering::Relaxed),
            events_sent: EVENTS_SENT.swap(0, Ordering::Relaxed),
            decode_errors: DECODE_ERRORS.swap(0, Ordering::Relaxed),
            send_errors: SEND_ERRORS.swap(0, Ordering::Relaxed),
            read_errors: READ_ERRORS.swap(0, Ordering::Relaxed),
        }
    }

    /// Errors in this interval — the number Sentry gets, instead of one event each
    fn errors(&self) -> u64 {
        self.decode_errors + self.send_errors + self.read_errors
    }

    /// Render the counters as a single line
    fn summary(&self, connections: usize, subscriptions: usize) -> String {
        let ready_avg_ms = self.ready_micros.checked_div(self.ready).unwrap_or(0) / 1000;

        format!(
            "gateway: {connections} connections (+{} accepted), {subscriptions} subscriptions, \
             {} ready (avg {ready_avg_ms}ms), {} events sent, \
             errors: {} decode / {} send / {} read",
            self.accepted,
            self.ready,
            self.events_sent,
            self.decode_errors,
            self.send_errors,
            self.read_errors
        )
    }
}

/// Log a summary every [`SUMMARY_INTERVAL`], forever
pub async fn report_forever() {
    loop {
        tokio::time::sleep(SUMMARY_INTERVAL).await;

        let interval = Interval::take();
        let summary = interval.summary(
            CONNECTIONS.load(Ordering::Relaxed),
            SUBSCRIPTIONS.load(Ordering::Relaxed),
        );

        info!("{summary}");

        if interval.errors() > 0 {
            sentry::capture_message(&summary, sentry::Level::Warning);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_is_reported_once_and_reset() {
        ACCEPTED.store(3, Ordering::Relaxed);
        READY.store(2, Ordering::Relaxed);
        READY_MICROS.store(5_000, Ordering::Relaxed);
        EVENTS_SENT.store(41, Ordering::Relaxed);
        DECODE_ERRORS.store(1, Ordering::Relaxed);

        let interval = Interval::take();
        assert_eq!(interval.accepted, 3);
        assert_eq!(interval.events_sent, 41);
        assert_eq!(interval.errors(), 1);

        let summary = interval.summary(7, 9);
        assert!(summary.contains("7 connections (+3 accepted)"), "{summary}");
        assert!(summary.contains("9 subscriptions"), "{summary}");
        // Two Ready payloads over 5000us is 2.5ms each, truncated to whole milliseconds.
        assert!(summary.contains("2 ready (avg 2ms)"), "{summary}");
        assert!(summary.contains("1 decode / 0 send / 0 read"), "{summary}");

        // A second report must not count the same interval again.
        assert_eq!(Interval::take(), Interval::default());
    }

    #[test]
    fn subscriptions_gauge_follows_a_connection() {
        SUBSCRIPTIONS.store(0, Ordering::Relaxed);

        let mut mine = 0;
        track_subscriptions(&mut mine, 5);
        assert_eq!(SUBSCRIPTIONS.load(Ordering::Relaxed), 5);

        track_subscriptions(&mut mine, 2);
        assert_eq!(SUBSCRIPTIONS.load(Ordering::Relaxed), 2);

        // Disconnecting leaves the gauge where it started, which is what makes it a gauge
        // and not a slow leak.
        track_subscriptions(&mut mine, 0);
        assert_eq!(SUBSCRIPTIONS.load(Ordering::Relaxed), 0);
        assert_eq!(mine, 0);
    }
}
