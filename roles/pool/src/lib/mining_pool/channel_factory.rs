use roles_logic_sv2::channel_logic::{ChannelFactory, ChannelFactoryError, ChannelFactoryTemplateDistribution, ChannelFactoryTemplateDistributionError};
use roles_logic_sv2::mining_sv2::{OpenStandardMiningChannel, OpenExtendedMiningChannel, UpdateChannel, NewExtendedMiningJob, NewMiningJob, SetNewPrevHash, SubmitSharesStandard, SubmitSharesExtended};
use roles_logic_sv2::template_distribution_sv2::{NewTemplate, SetNewPrevHash as SetNewPrevHashTDP};
use roles_logic_sv2::parsers::Mining;
use roles_logic_sv2::utils::Id;
use roles_logic_sv2::mining_sv2::ExtendedExtranonce;
use nohash_hasher::BuildNoHashHasher;
use std::collections::HashMap;
use std::ops::Range;
#[derive(Clone, Debug)]
pub struct PoolChannelManager {
    id_factory: Id,
    extended_extranonce: ExtendedExtranonce,
}

impl PoolChannelManager {
    pub fn new() -> Self {
        Self { 
            id_factory: Id::new(),
            // todo: additional_coinbase_script_data
            extended_extranonce: ExtendedExtranonce::new(Range { start: 0, end: 0 }, Range { start: 0, end: 16 }, Range { start: 16, end: 32 }),
        }
    }

    pub fn next_id(&mut self) -> u32 {
        self.id_factory.next()
    }
}

impl ChannelFactory for PoolChannelManager {
    fn on_open_standard_mining_channel(&mut self, m: OpenStandardMiningChannel) -> Result<(), ChannelFactoryError> {
        todo!()
    }

    fn on_open_extended_mining_channel(&mut self, m: OpenExtendedMiningChannel) -> Result<(), ChannelFactoryError> {
        todo!()
    }

    fn on_update_channel(&mut self, m: UpdateChannel) -> Result<(), ChannelFactoryError> {
        todo!()
    }

    fn on_submit_shares_standard(&mut self, m: SubmitSharesStandard) -> Result<(), ChannelFactoryError> {
        todo!()
    }

    fn on_submit_shares_extended(&mut self, m: SubmitSharesExtended) -> Result<(), ChannelFactoryError> {
        todo!()
    }
}

impl ChannelFactoryTemplateDistribution for PoolChannelManager {
    fn on_set_new_prev_hash_tdp(&mut self, m: SetNewPrevHashTDP) -> Result<u32, ChannelFactoryTemplateDistributionError> {
        todo!()
    }

    fn on_new_template(&mut self, m: NewTemplate) -> Result<HashMap<u32, Mining<'static>, BuildNoHashHasher<u32>>, ChannelFactoryTemplateDistributionError> {
        todo!()
    }
}
