//! # Channel Logic
//!
//! A module for managing channels on applications.
//!
//! Divided in two submodules:
//! - [`channel_factory`]
//! - [`proxy_group_channel`]

// pub mod channel_factory;
pub mod proxy_group_channel;

use crate::parsers::Mining;
use template_distribution_sv2::{NewTemplate, SetNewPrevHash as SetNewPrevHashTDP};
use mining_sv2::{OpenStandardMiningChannel, OpenExtendedMiningChannel, UpdateChannel, NewExtendedMiningJob, NewMiningJob, SetNewPrevHash, SubmitSharesStandard, SubmitSharesExtended};
use nohash_hasher::BuildNoHashHasher;
use std::collections::HashMap;
use std::convert::TryInto;
use binary_sv2::U256;

/// Convert extended to standard job by calculating the merkle root
pub fn extended_to_standard_job<'a>(
    extended: &NewExtendedMiningJob,
    coinbase_script: &[u8],
    channel_id: u32,
    job_id: Option<u32>,
) -> Option<NewMiningJob<'a>> {
    let merkle_root = crate::utils::merkle_root_from_path(
        extended.coinbase_tx_prefix.inner_as_ref(),
        extended.coinbase_tx_suffix.inner_as_ref(),
        coinbase_script,
        &extended.merkle_path.inner_as_ref(),
    );

    Some(NewMiningJob {
        channel_id,
        job_id: job_id.unwrap_or(extended.job_id),
        min_ntime: extended.min_ntime.clone().into_static(),
        version: extended.version,
        merkle_root: merkle_root?.try_into().ok()?,
    })
}

/// ------------------------------------------------------------------------------------------------

// standard channels can receive extended jobs via group channels
// if the downstream is HOM, the extended job received from group channel is converted to a standard job before being sent downstream
// if the downstream is non-HOM, the extended job is propagated as is to the downstream group channel
pub enum StandardOrExtendedJob<'decoder> {
    Standard(NewMiningJob<'decoder>),
    ExtendedFromGroup(NewExtendedMiningJob<'decoder>),
}

pub trait StandardChannel {
    fn get_channel_id(&self) -> u32;
    fn get_group_channel_id(&self) -> u32;
    fn get_user_identity(&self) -> &str;
    fn get_extranonce_prefix(&self) -> u32;
    fn get_target(&self) -> U256;
    fn get_nominal_hashrate(&self) -> f32;
    fn get_prev_hash(&self) -> U256;
    fn get_current_job(&self) -> Option<StandardOrExtendedJob<'_>>;
    fn get_future_job(&self) -> Option<StandardOrExtendedJob<'_>>;
}

pub trait GroupChannel<S: StandardChannel> {
    fn get_channel_id(&self) -> u32;
    fn get_channels(&self) -> Vec<S>;
    fn add_channel(&mut self, channel: S);
    fn remove_channel(&mut self, channel: S);
    fn get_current_job(&self) -> Option<NewExtendedMiningJob>;
    fn get_future_job(&self) -> Option<NewExtendedMiningJob>;
}

pub trait ExtendedChannel {
    fn get_channel_id(&self) -> u32;
    fn get_user_identity(&self) -> &str;
    fn get_extranonce_prefix(&self) -> u32;
    fn get_extranonce_size(&self) -> u16;
    fn get_target(&self) -> U256;
    fn get_nominal_hashrate(&self) -> f32;
    fn get_prev_hash(&self) -> U256;
    fn get_current_job(&self) -> Option<NewExtendedMiningJob>;
    fn get_future_job(&self) -> Option<NewExtendedMiningJob>;
}

pub trait StandardChannelFactory<S: StandardChannel, G: GroupChannel<S>> {
    fn get_group_channel(&self, group_channel_id: u32) -> G;
    fn on_open_standard_mining_channel(&mut self, m: OpenStandardMiningChannel) -> Result<(), StandardChannelFactoryError>;
    fn on_update_channel(&mut self, m: UpdateChannel) -> Result<(), StandardChannelFactoryError>;
    fn on_submit_shares_standard(&mut self, m: SubmitSharesStandard) -> Result<(), StandardChannelFactoryError>;
    // fn check_target(&mut self, m: Share) -> Result<(), ChannelFactoryError>;
}

#[derive(Debug)]
pub enum StandardChannelFactoryError {
}

pub trait ExtendedChannelFactory<E: ExtendedChannel> {
    fn on_open_extended_mining_channel(&mut self, m: OpenExtendedMiningChannel) -> Result<(), ExtendedChannelFactoryError>;
    fn on_update_channel(&mut self, m: UpdateChannel) -> Result<(), ExtendedChannelFactoryError>;
    fn on_submit_shares_extended(&mut self, m: SubmitSharesExtended) -> Result<(), ExtendedChannelFactoryError>;
}

pub enum ExtendedChannelFactoryError {
}

pub trait StandardChannelFactoryTemplateDistribution<S: StandardChannel, G: GroupChannel<S>>: StandardChannelFactory<S, G> {
    fn on_set_new_prev_hash_tdp(&mut self, m: SetNewPrevHashTDP) -> Result<(), StandardChannelFactoryTemplateDistributionError>;
    fn on_new_template(&mut self, m: NewTemplate) -> Result<(), StandardChannelFactoryTemplateDistributionError>;
}

#[derive(Debug)]
pub enum StandardChannelFactoryTemplateDistributionError {
}

pub trait ExtendedChannelFactoryTemplateDistribution<E: ExtendedChannel>: ExtendedChannelFactory<E> {
    fn on_set_new_prev_hash_tdp(&mut self, m: SetNewPrevHashTDP) -> Result<(), ExtendedChannelFactoryTemplateDistributionError>;
    fn on_new_template(&mut self, m: NewTemplate) -> Result<(), ExtendedChannelFactoryTemplateDistributionError>;
}

pub enum ExtendedChannelFactoryTemplateDistributionError {
}