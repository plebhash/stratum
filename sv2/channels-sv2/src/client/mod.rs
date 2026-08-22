//! Sv2 channels - Mining Clients Abstractions.
//!
//! The `client` module is compatible with `no_std` environments. To enable this mode, build the
//! crate with the `no_std` feature. In this configuration, standard library collections are
//! replaced with the `hashbrown` crate, together with `core` and `alloc`, allowing the module to be
//! used in embedded or constrained contexts.

pub mod error;
pub mod extended;
pub mod group;
pub mod share_accounting;
pub mod standard;

/// Maximum number of future jobs a client channel retains while waiting for a
/// [`SetNewPrevHash`](mining_sv2::SetNewPrevHash) (or chain tip update).
///
/// Upstream servers control `job_id`, so future jobs are stored under an upstream-controlled key.
/// Bounding this map prevents a malicious or buggy server from exhausting client memory by
/// streaming future jobs while withholding [`SetNewPrevHash`](mining_sv2::SetNewPrevHash). On
/// overflow, the oldest future job is evicted.
pub const MAX_FUTURE_JOBS: usize = 16;

/// Maximum number of past jobs a client channel retains under the current chain tip.
///
/// Upstream servers control the job stream, so a malicious or buggy server can force one retained
/// past job per immediately-active job message. Bounding this map prevents unbounded memory
/// growth. Past jobs exist for late-share validation, so the cap must stay nonzero. On overflow,
/// the oldest past job is evicted: a share against it is rejected as
/// [`InvalidJobId`](crate::client::share_accounting::ShareValidationError::InvalidJobId) even
/// though it would otherwise have been accepted and propagated — a bounded loss of creditable
/// work, the price of bounding memory under a hostile upstream.
///
/// Matches the server-side default, so a proxy's client channel never evicts a job its upstream
/// still accepts. A client retaining more than its upstream gains nothing: the proxy credits the
/// share locally and the upstream rejects it anyway.
///
/// The cap is really a retention *window* — `cap / job rate` — so the count a deployment needs
/// depends on how fast the upstream sends new jobs, which this crate cannot see. Hardware
/// measurement bounded the requirement at **~16 s of retention**, and 16 is that window at one job
/// per second, the fastest rate an upstream's own job production can be configured for. The miner
/// A/B runs and the production share-age survey behind that bound are in the comments on
/// [PR #2307](https://github.com/stratum-mining/stratum/pull/2307).
///
/// Operators who know their upstream's job interval `T` should set `ceil(16 s / T)` and reclaim
/// the memory: 3 at a typical 6 s interval costs 13.5 kB per channel against 72 kB for 16. That
/// matters most on a translator, where every downstream miner holds its own client channel. On a
/// job-declaration client the upstream channel's job rate is instead the client's own
/// `SetCustomMiningJob` rate, which it does know.
///
/// Channel constructors accept a `max_past_jobs` override, falling back to this value on
/// `None`/`Some(0)`. See [PR #2290](https://github.com/stratum-mining/stratum/pull/2290) for the
/// memory cost.
pub const MAX_PAST_JOBS: usize = 16;

/// Maximum number of accepted-share hashes a client channel retains for duplicate detection.
///
/// 4 096 hashes is one 128 KB allocation, which keeps the `no_std`/embedded use case viable:
/// the bound has to be affordable on the smallest supported device, since an adversarial
/// upstream advertising a trivial target can drive the cache to it at message speed.
///
/// A client cache does not need to hold a whole chain tip's worth of shares. It exists to catch
/// a share source re-submitting work it already sent — a retransmit or a buggy loop, which
/// arrives within seconds — not to reconcile a tip. 4 096 covers ~11 hours of history for a
/// typical 6 shares/min channel and ~7 minutes for a very busy 600 shares/min proxy channel,
/// far beyond any realistic duplicate window in both cases. Overflow evicts oldest-first, and
/// an evicted-then-replayed hash costs one double-counted local statistic.
pub const MAX_SEEN_SHARES: usize = 4_096;

// Type aliases that switch between `std::collections` and `hashbrown`
// depending on whether the `no_std` feature is enabled.
#[cfg(not(feature = "no_std"))]
type HashMap<K, V> = std::collections::HashMap<K, V>;
#[cfg(not(feature = "no_std"))]
type HashSet<T> = std::collections::HashSet<T>;
#[cfg(feature = "no_std")]
type HashMap<K, V> = hashbrown::HashMap<K, V>;
#[cfg(feature = "no_std")]
type HashSet<T> = hashbrown::HashSet<T>;
