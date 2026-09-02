//! Internal job storage and lifecycle management for server-side SV2 channels.
//!
//! ## Responsibilities
//!
//! - **Job Storage**: Manages collections of jobs indexed by job ID and template ID.
//! - **Job Activation**: Handles transitions between future, active, past, and stale jobs.
//! - **Template Mapping**: Tracks mappings from template IDs to job IDs for future jobs.
//! - **Lifecycle Management**: Ensures correct state transitions when activating jobs or updating
//!   chain tips.
//! - **Retired Extranonce Prefixes**: Holds on to extranonce prefixes that were rotated out of the
//!   channel while jobs created under them can still accept shares, so that their allocator slots
//!   are not handed to another channel too early.
use std::collections::{HashMap, VecDeque};

use super::Job;
use crate::extranonce_manager::ExtranoncePrefix;

/// Maximum number of future jobs a server channel retains while waiting for a
/// template-distribution `SetNewPrevHash`.
///
/// Template Distribution peers control `template_id`, so future jobs are stored under a
/// peer-controlled key. Bounding the map prevents a malicious or buggy peer from exhausting
/// server memory by streaming future templates while withholding `SetNewPrevHash`. On overflow,
/// the oldest future job is evicted.
pub(crate) const MAX_FUTURE_JOBS: usize = 16;

/// Maximum number of past jobs a server channel retains under the current chain tip.
///
/// Past jobs serve late-share validation, so the cap must stay nonzero. On overflow the oldest is
/// evicted and a share against it is rejected as `InvalidJobId` — a bounded loss of creditable
/// work, the price of bounding memory against a hostile peer.
///
/// The cap is a retention *window* — `cap / job rate` — and the rate belongs to the deployment.
/// Measurement bounded the requirement at ~16 s (PR #2307), so 16 covers the fastest configurable
/// rate of one job per second. Operators who know their own interval `T` should set
/// `ceil(16 s / T)`: 3 at a typical 6 s interval, 13.5 kB per channel against 72 kB (PR #2290).
///
/// Channels accepting `SetCustomMiningJob` are the exception — each accepted job retires the
/// active one, so the rate is the client's. Size against the rate the pool permits, or keep 16.
///
/// Constructors take a `max_past_jobs` override, falling back here on `None`/`Some(0)`.
pub(crate) const MAX_PAST_JOBS: usize = 16;

/// Internal implementation for tracking mining job states in SV2 server channels.
///
/// Maintains collections for future, active, past, and stale jobs, and tracks template-to-job ID
/// mappings for future job activation.
#[derive(Debug)]
pub(crate) struct JobStore<T: Job> {
    future_template_to_job_id: HashMap<u64, u32>,
    // Future template IDs ordered by receipt, oldest at the front and newest at the back.
    // Replaced IDs move to the back; overflow evicts from the front.
    future_template_order: VecDeque<u64>,
    // Future jobs are indexed with job_id (u32)
    future_jobs: HashMap<u32, T>,
    active_job: Option<T>,
    // Past jobs are indexed with job_id (u32)
    past_jobs: HashMap<u32, T>,
    // Past job IDs ordered by retirement, oldest at the front and newest at the back.
    // Replaced IDs move to the back; overflow evicts from the front.
    past_job_order: VecDeque<u32>,
    // Stale jobs are indexed with job_id (u32)
    stale_jobs: HashMap<u32, T>,
    // Extranonce prefixes rotated out of the channel that are still referenced by at least one job
    // that can accept shares. Holding the object here keeps its allocator slot reserved; dropping
    // it releases the slot.
    retired_extranonce_prefixes: Vec<ExtranoncePrefix>,
    // Cap on `past_jobs` under the current chain tip. Nonzero by construction: channel
    // constructors resolve `None`/`Some(0)` to `MAX_PAST_JOBS` before reaching here.
    max_past_jobs: usize,
}

impl<T: Job> JobStore<T> {
    /// Creates a new empty job store retaining at most `max_past_jobs` past jobs under the
    /// current chain tip.
    pub fn new(max_past_jobs: usize) -> Self {
        // channel constructors resolve `None`/`Some(0)` to `MAX_PAST_JOBS`, so a zero cap cannot
        // arrive here; it would evict the just-retired job immediately and reject the most common
        // late share as `InvalidJobId`
        debug_assert!(max_past_jobs > 0, "max_past_jobs must be nonzero");
        Self {
            future_template_to_job_id: HashMap::new(),
            future_template_order: VecDeque::new(),
            future_jobs: HashMap::new(),
            active_job: None,
            past_jobs: HashMap::new(),
            past_job_order: VecDeque::new(),
            stale_jobs: HashMap::new(),
            retired_extranonce_prefixes: Vec::new(),
            max_past_jobs,
        }
    }
}

impl<T: Job> Default for JobStore<T> {
    fn default() -> Self {
        Self::new(MAX_PAST_JOBS)
    }
}

impl<T: Job> JobStore<T> {
    /// Adds a future job associated with a template ID.
    ///
    /// If the template ID was already mapped to a future job, that job is dropped, since it could
    /// never be activated again (activation resolves jobs through this mapping).
    ///
    /// At most `MAX_FUTURE_JOBS` future jobs are kept: storing a new one beyond that limit evicts
    /// the oldest, since template IDs are peer-controlled and must not grow memory unboundedly.
    ///
    /// Returns the new job's ID.
    pub fn add_future_job(&mut self, template_id: u64, new_job: T) -> u32 {
        let mut dropped_job = false;

        let new_job_id = new_job.get_job_id();
        if let Some(old_job_id) = self
            .future_template_to_job_id
            .insert(template_id, new_job_id)
        {
            self.future_jobs.remove(&old_job_id);
            dropped_job = true;
        }
        self.future_jobs.insert(new_job_id, new_job);

        // a replaced template_id moves to the back of the eviction order
        self.future_template_order.retain(|id| *id != template_id);
        self.future_template_order.push_back(template_id);

        if self.future_jobs.len() > MAX_FUTURE_JOBS {
            if let Some(evicted_template_id) = self.future_template_order.pop_front() {
                if let Some(evicted_job_id) =
                    self.future_template_to_job_id.remove(&evicted_template_id)
                {
                    self.future_jobs.remove(&evicted_job_id);
                    dropped_job = true;
                }
            }
        }

        // a dropped job (replaced template ID or evicted-oldest) may have been the last one
        // holding a retired extranonce prefix alive; release such slots now rather than at the
        // next chain transition, which a peer can withhold
        if dropped_job {
            self.prune_retired_extranonce_prefixes();
        }

        new_job_id
    }

    /// Moves the active job (if any) into past jobs, evicting the oldest past job beyond this
    /// store's `max_past_jobs`. A share against an evicted job is rejected as `InvalidJobId` even
    /// though it would otherwise have been accepted and credited: a bounded loss of creditable
    /// work, the price of bounding memory under a hostile upstream.
    ///
    /// Returns the evicted job's ID, if any, so callers can drop metadata they key by job ID.
    fn retire_active_to_past(&mut self) -> Option<u32> {
        let active_job = self.active_job.take()?;
        let job_id = active_job.get_job_id();
        self.past_jobs.insert(job_id, active_job);

        // job IDs are minted by the job factory, strictly monotonic per channel, so a retiring
        // ID can never already be in the eviction order
        self.past_job_order.push_back(job_id);

        if self.past_jobs.len() > self.max_past_jobs {
            if let Some(evicted_job_id) = self.past_job_order.pop_front() {
                self.past_jobs.remove(&evicted_job_id);
                // the evicted job may have been the last one holding a retired extranonce
                // prefix alive; release such slots now rather than at the next chain
                // transition, which a peer can withhold
                self.prune_retired_extranonce_prefixes();
                return Some(evicted_job_id);
            }
        }
        None
    }

    /// Moves the active job (if any) into past jobs without the `max_past_jobs` cap and without
    /// pruning retired extranonce prefixes.
    ///
    /// Only for tip transitions, where past jobs immediately drain into stale jobs: `stale_jobs`
    /// stays bounded at `max_past_jobs + 1`, no job is dropped from the stale set (which would
    /// misclassify its late shares as `InvalidJobId` instead of `Stale`), and no prune runs
    /// while an in-flight future job is outside every collection (which would release its
    /// retired extranonce prefix while the job goes on to accept shares under it).
    fn retire_active_to_past_uncapped(&mut self) {
        if let Some(active_job) = self.active_job.take() {
            let job_id = active_job.get_job_id();
            self.past_jobs.insert(job_id, active_job);

            // job IDs are minted by the job factory, strictly monotonic per channel, so a
            // retiring ID can never already be in the eviction order
            self.past_job_order.push_back(job_id);
        }
    }

    /// Adds an active job, moving the previous active job (if any) to past jobs.
    ///
    /// At most `max_past_jobs` past jobs are kept: retiring one beyond that limit evicts the
    /// oldest (giving up shares that would still have been creditable), and its ID is returned
    /// so callers can drop metadata they key by job ID (e.g. the per-job target mapping of
    /// standard and extended channels).
    pub fn add_active_job(&mut self, job: T) -> Option<u32> {
        // Move currently active job to past jobs (so it can be marked as stale)
        let evicted_job_id = self.retire_active_to_past();
        // Set the new active job
        self.active_job = Some(job);
        evicted_job_id
    }

    /// Replaces the active job, dropping the previous active job (if any).
    ///
    /// For channels that never validate shares (group channels), where retaining the replaced
    /// job would be pure memory growth under peer-controlled message streams.
    pub fn replace_active_job(&mut self, job: T) {
        self.active_job = Some(job);
    }

    /// Activates a future job given by template ID and header timestamp, dropping the previously
    /// active job (if any) instead of keeping it as stale.
    /// Returns `true` if successful, `false` if not found.
    ///
    /// For channels that never validate shares (group channels), which keep no past or stale
    /// job history. A failed activation leaves channel state untouched.
    pub fn activate_future_job_replacing_active(
        &mut self,
        template_id: u64,
        prev_hash_header_timestamp: u32,
    ) -> bool {
        let activated = self.activate_future_job(template_id, prev_hash_header_timestamp);
        if activated {
            // group channels keep no job history: the job displaced by this activation went
            // stale above and is dropped here
            self.stale_jobs.clear();
        }
        activated
    }

    /// Activates a future job given by template ID and header timestamp.
    /// Returns `true` if successful, `false` if not found.
    pub fn activate_future_job(
        &mut self,
        template_id: u64,
        prev_hash_header_timestamp: u32,
    ) -> bool {
        let mut future_job =
            if let Some(job_id) = self.future_template_to_job_id.remove(&template_id) {
                if let Some(job) = self.future_jobs.remove(&job_id) {
                    job
                } else {
                    return false;
                }
            } else {
                return false;
            };

        // Move currently active job to past jobs (so it can be marked as stale). The
        // retirement is uncapped: past jobs drain into stale jobs below, so the displaced job
        // must not push another one out of the stale set, and no prune may run while the
        // in-flight future job is outside every collection.
        self.retire_active_to_past_uncapped();

        // Activate the future job
        future_job.activate(prev_hash_header_timestamp);
        self.active_job = Some(future_job);
        self.future_jobs.clear();
        self.future_template_to_job_id.clear();
        self.future_template_order.clear();

        self.mark_past_jobs_as_stale();

        true
    }

    /// Moves the active job (if any) into past jobs, without the `max_past_jobs` cap.
    ///
    /// Only for tip transitions, right before [`Self::mark_past_jobs_as_stale`] drains past
    /// jobs into stale jobs: the displaced job must go stale with the rest, not push another
    /// job out of the stale set.
    pub fn deactivate_job(&mut self) {
        self.retire_active_to_past_uncapped();
    }

    /// Drops the active job (if any) without retaining it.
    ///
    /// For channels that never validate shares against stored jobs (group channels), routing
    /// the job through the past → stale rotation would only retain state that can never be
    /// referenced again.
    pub fn clear_active_job(&mut self) {
        self.active_job = None;
    }

    /// Marks all past jobs as stale so shares can be rejected with the proper error code.
    pub fn mark_past_jobs_as_stale(&mut self) {
        // Transfer past jobs to stale jobs collection and reset past jobs to empty
        self.stale_jobs = std::mem::take(&mut self.past_jobs);
        self.past_job_order.clear();
        // jobs that just went stale can no longer accept shares, so any retired extranonce prefix
        // they were the last reference to is now releasable
        self.prune_retired_extranonce_prefixes();
    }

    /// Takes ownership of an extranonce prefix that is no longer the channel's current one,
    /// releasing it only once no job created under it can accept shares anymore.
    ///
    /// Jobs only carry a copy of the extranonce prefix bytes they were created under, and stay
    /// valid across a prefix rotation. Dropping the prefix object right away would return its
    /// slot to the allocator while those jobs still validate shares under those bytes, allowing
    /// the same extranonce space to be handed to a second live channel.
    ///
    /// Only prefixes that [hold a slot](ExtranoncePrefix::holds_allocator_slot) are ever kept:
    /// wire-sourced prefixes and allocator-produced ones whose allocator is gone reserve nothing,
    /// and retaining them would let byte-identical rotations grow the retained set once per
    /// update. Previously retired prefixes whose allocator has since been dropped are released
    /// here as well.
    pub fn retire_extranonce_prefix(&mut self, extranonce_prefix: ExtranoncePrefix) {
        self.retired_extranonce_prefixes
            .retain(ExtranoncePrefix::holds_allocator_slot);
        if extranonce_prefix.holds_allocator_slot()
            && self.is_extranonce_prefix_in_use(extranonce_prefix.as_bytes())
        {
            self.retired_extranonce_prefixes.push(extranonce_prefix);
        }
        // otherwise it drops here, releasing its slot right away
    }

    /// Drops every retired extranonce prefix that no future, active or past job still references
    /// (or whose allocator has since been dropped). Dropping releases the prefix's slot back to
    /// its allocator.
    ///
    /// Stale jobs are deliberately not consulted: shares against them are rejected as stale, so
    /// they can no longer be credited under the old prefix bytes.
    fn prune_retired_extranonce_prefixes(&mut self) {
        // bind the job collections separately, so that the closure borrows them instead of
        // borrowing all of `self` (which `retain` needs mutably)
        let future_jobs = &self.future_jobs;
        let active_job = &self.active_job;
        let past_jobs = &self.past_jobs;

        self.retired_extranonce_prefixes.retain(|prefix| {
            prefix.holds_allocator_slot()
                && future_jobs
                    .values()
                    .chain(active_job.iter())
                    .chain(past_jobs.values())
                    .any(|job| job.get_extranonce_prefix() == prefix.as_bytes())
        });
    }

    /// Whether any future, active or past job was created under `extranonce_prefix`.
    fn is_extranonce_prefix_in_use(&self, extranonce_prefix: &[u8]) -> bool {
        self.future_jobs
            .values()
            .chain(self.active_job.iter())
            .chain(self.past_jobs.values())
            .any(|job| job.get_extranonce_prefix() == extranonce_prefix)
    }

    /// Returns the job ID for a future job from a template ID, if any.
    pub fn get_future_job_id_from_template_id(&self, template_id: u64) -> Option<u32> {
        self.future_template_to_job_id.get(&template_id).cloned()
    }

    /// Returns a reference to the currently active job, if any.
    pub fn get_active_job(&self) -> Option<&T> {
        self.active_job.as_ref()
    }

    /// Returns true if there are any future jobs, false otherwise.
    pub fn has_future_jobs(&self) -> bool {
        !self.future_jobs.is_empty()
    }

    /// Returns a reference to a future job from its job ID, if any.
    pub fn get_future_job(&self, job_id: u32) -> Option<&T> {
        self.future_jobs.get(&job_id)
    }

    /// Returns a reference to a past job from its job ID, if any.
    pub fn get_past_job(&self, job_id: u32) -> Option<&T> {
        self.past_jobs.get(&job_id)
    }

    /// Returns a reference to a stale job from its job ID, if any.
    pub fn get_stale_job(&self, job_id: u32) -> Option<&T> {
        self.stale_jobs.get(&job_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extranonce_manager::ExtranonceAllocator;

    struct DummyJob {
        job_id: u32,
    }

    impl Job for DummyJob {
        fn get_job_id(&self) -> u32 {
            self.job_id
        }

        fn get_extranonce_prefix(&self) -> &[u8] {
            &[]
        }

        fn activate(&mut self, _prev_hash_header_timestamp: u32) {}
    }

    #[test]
    fn future_jobs_are_bounded() {
        let mut store = JobStore::new(MAX_PAST_JOBS);

        let flood_size = 10_000u64;
        for template_id in 0..flood_size {
            store.add_future_job(
                template_id,
                DummyJob {
                    job_id: template_id as u32,
                },
            );
        }

        // only the newest MAX_FUTURE_JOBS survive; the oldest were evicted
        for template_id in 0..flood_size - MAX_FUTURE_JOBS as u64 {
            assert!(store
                .get_future_job_id_from_template_id(template_id)
                .is_none());
            assert!(store.get_future_job(template_id as u32).is_none());
        }
        for template_id in flood_size - MAX_FUTURE_JOBS as u64..flood_size {
            assert!(store
                .get_future_job_id_from_template_id(template_id)
                .is_some());
            assert!(store.get_future_job(template_id as u32).is_some());
        }
    }

    #[test]
    fn past_jobs_respect_a_custom_cap() {
        // a store built with a cap below the default must evict against that cap, not the default
        let custom_cap = 2;
        assert!(custom_cap < MAX_PAST_JOBS);
        let mut store = JobStore::new(custom_cap);

        for job_id in 0..10u32 {
            let evicted_job_id = store.add_active_job(DummyJob { job_id });
            if job_id as usize > custom_cap {
                assert_eq!(evicted_job_id, Some(job_id - custom_cap as u32 - 1));
            } else {
                assert_eq!(evicted_job_id, None);
            }
        }

        assert_eq!(store.past_jobs.len(), custom_cap);
        assert_eq!(store.past_job_order.len(), custom_cap);
        // job 9 is active; only the two most recently retired survive
        assert!(store.get_past_job(8).is_some());
        assert!(store.get_past_job(7).is_some());
        for job_id in 0..7u32 {
            assert!(store.get_past_job(job_id).is_none());
        }
    }

    #[test]
    fn default_job_store_uses_the_default_cap() {
        let store: JobStore<DummyJob> = JobStore::default();
        assert_eq!(store.max_past_jobs, MAX_PAST_JOBS);
    }

    #[test]
    fn past_jobs_are_bounded() {
        let mut store = JobStore::new(MAX_PAST_JOBS);

        let flood_size = 10_000u32;
        for job_id in 0..flood_size {
            let evicted_job_id = store.add_active_job(DummyJob { job_id });
            // the first eviction happens once MAX_PAST_JOBS past jobs already exist; from
            // then on each retirement evicts the oldest and reports its ID
            if job_id as usize > MAX_PAST_JOBS {
                assert_eq!(evicted_job_id, Some(job_id - MAX_PAST_JOBS as u32 - 1));
            } else {
                assert_eq!(evicted_job_id, None);
            }
        }

        // the last job is active; of the retired ones, only the newest MAX_PAST_JOBS survive
        for job_id in 0..flood_size - 1 - MAX_PAST_JOBS as u32 {
            assert!(store.get_past_job(job_id).is_none());
        }
        for job_id in flood_size - 1 - MAX_PAST_JOBS as u32..flood_size - 1 {
            assert!(store.get_past_job(job_id).is_some());
        }
        assert_eq!(
            store.get_active_job().map(|job| job.get_job_id()),
            Some(flood_size - 1)
        );
    }

    struct PrefixedJob {
        job_id: u32,
        prefix: Vec<u8>,
    }

    impl Job for PrefixedJob {
        fn get_job_id(&self) -> u32 {
            self.job_id
        }

        fn get_extranonce_prefix(&self) -> &[u8] {
            &self.prefix
        }

        fn activate(&mut self, _prev_hash_header_timestamp: u32) {}
    }

    #[test]
    fn tip_transition_moves_displaced_and_all_past_jobs_to_stale() {
        let mut store = JobStore::new(MAX_PAST_JOBS);

        // fill past jobs to the cap, plus an active job
        for job_id in 0..=MAX_PAST_JOBS as u32 {
            store.add_active_job(DummyJob { job_id });
        }

        let future_job_id = 100;
        store.add_future_job(
            1,
            DummyJob {
                job_id: future_job_id,
            },
        );
        assert!(store.activate_future_job(1, 0));

        // the displaced active job and every past job must land in the stale set: dropping
        // any of them would misclassify its late shares as InvalidJobId instead of Stale
        for job_id in 0..=MAX_PAST_JOBS as u32 {
            assert!(store.get_stale_job(job_id).is_some());
        }
        assert_eq!(store.stale_jobs.len(), MAX_PAST_JOBS + 1);
        assert!(store.past_jobs.is_empty());
        assert_eq!(
            store.get_active_job().map(|job| job.get_job_id()),
            Some(future_job_id)
        );
    }

    #[test]
    fn activation_keeps_retired_prefix_of_activated_job() {
        let mut store = JobStore::new(MAX_PAST_JOBS);
        // the allocator outlives the test, so the retired prefix keeps holding its slot
        let mut allocator = ExtranonceAllocator::new(vec![], 1, 2).unwrap();
        let old_prefix = allocator.allocate_extended(0).unwrap();

        // a future job created under the old prefix, which is then rotated out
        store.add_future_job(
            1,
            PrefixedJob {
                job_id: 100,
                prefix: old_prefix.as_bytes().to_vec(),
            },
        );
        store.retire_extranonce_prefix(old_prefix.into());
        assert_eq!(store.retired_extranonce_prefixes.len(), 1);

        // fill past jobs to the cap with jobs under the new prefix, so that a capped
        // retirement during activation would evict (and prune) mid-flight
        for job_id in 0..=MAX_PAST_JOBS as u32 {
            store.add_active_job(PrefixedJob {
                job_id,
                prefix: vec![2u8],
            });
        }

        assert!(store.activate_future_job(1, 0));

        // the activated job is the only remaining reference to the retired prefix; its slot
        // must stay reserved while the job keeps accepting shares under those bytes
        assert_eq!(store.retired_extranonce_prefixes.len(), 1);
    }

    #[test]
    fn dropped_jobs_release_retired_extranonce_prefixes() {
        let mut store = JobStore::new(MAX_PAST_JOBS);
        // the allocator outlives the test, so the retired prefix keeps holding its slot
        let mut allocator = ExtranonceAllocator::new(vec![], 1, 2).unwrap();
        let old_prefix = allocator.allocate_extended(0).unwrap();

        // a future job created under the old prefix keeps the retired prefix alive
        store.add_future_job(
            1,
            PrefixedJob {
                job_id: 1,
                prefix: old_prefix.as_bytes().to_vec(),
            },
        );
        store.retire_extranonce_prefix(old_prefix.into());
        assert_eq!(store.retired_extranonce_prefixes.len(), 1);

        // replacing the future job under the same template ID drops the last job referencing
        // the retired prefix, so its slot must be released without waiting for a chain
        // transition
        store.add_future_job(
            1,
            PrefixedJob {
                job_id: 2,
                prefix: vec![2u8],
            },
        );
        assert!(store.retired_extranonce_prefixes.is_empty());
    }

    #[test]
    fn reused_template_id_evicts_superseded_future_job() {
        let mut store = JobStore::new(MAX_PAST_JOBS);

        let old_job_id = store.add_future_job(1, DummyJob { job_id: 10 });
        let new_job_id = store.add_future_job(1, DummyJob { job_id: 11 });

        // the superseded job could never be activated again, so it must not be retained
        assert!(store.get_future_job(old_job_id).is_none());
        assert!(store.get_future_job(new_job_id).is_some());
        assert_eq!(
            store.get_future_job_id_from_template_id(1),
            Some(new_job_id)
        );
    }

    #[test]
    fn dead_allocation_tokens_are_not_retained() {
        // A retired prefix whose allocator is gone reserves nothing, so retaining it would only
        // let byte-identical rotations (fresh allocators minting the same bytes) grow the retired
        // set once per update while one matching job stays alive.
        let prefix = vec![0u8];
        let mut store = JobStore::new(MAX_PAST_JOBS);
        store.add_active_job(PrefixedJob {
            job_id: 1,
            prefix: prefix.clone(),
        });

        for _ in 0..10_000 {
            let mut allocator = ExtranonceAllocator::new(vec![], 1, 2).unwrap();
            let allocated = allocator.allocate_extended(0).unwrap();
            assert_eq!(allocated.as_bytes(), prefix.as_slice());
            // the token still holds its slot when retired; its allocator goes away right after
            store.retire_extranonce_prefix(allocated.into());
        }
        // at most the last token survives: every earlier one was dead by the next retirement
        assert_eq!(store.retired_extranonce_prefixes.len(), 1);

        // a token whose allocator is already gone when it is retired is never kept, and neither
        // is a wire-sourced prefix (which never held a slot)
        let allocated = ExtranonceAllocator::new(vec![], 1, 2)
            .unwrap()
            .allocate_extended(0)
            .unwrap();
        store.retire_extranonce_prefix(allocated.into());
        assert!(store.retired_extranonce_prefixes.is_empty());
        store.retire_extranonce_prefix(ExtranoncePrefix::from_wire(prefix).unwrap());
        assert!(store.retired_extranonce_prefixes.is_empty());
    }
}
