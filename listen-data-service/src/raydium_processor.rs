use carbon_core::{
    account::AccountProcessorInputType, error::CarbonResult, metrics::MetricsCollection,
    processor::Processor,
};
use carbon_raydium_amm_v4_decoder::accounts::RaydiumAmmV4Account;
use std::sync::Arc;
use tracing::info;

pub struct RaydiumAmmV4AccountProcessor {}

impl RaydiumAmmV4AccountProcessor {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl Processor for RaydiumAmmV4AccountProcessor {
    type InputType = AccountProcessorInputType<RaydiumAmmV4Account>;

    async fn process(
        &mut self,
        data: Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let (_meta, account) = data;
        match &account.data {
            RaydiumAmmV4Account::AmmInfo(pool) => {
                info!("pool: {:#?}", pool);
            }
            _ => {}
        }

        Ok(())
    }
}
