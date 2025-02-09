use anyhow::Result;
use carbon_core::pipeline::Pipeline;
use carbon_log_metrics::LogMetrics;
use carbon_raydium_amm_v4_decoder::RaydiumAmmV4Decoder;
use carbon_yellowstone_grpc_datasource::YellowstoneGrpcGeyserClient;
use solana_sdk::{pubkey, pubkey::Pubkey};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::RwLock;
use yellowstone_grpc_proto::geyser::{
    CommitmentLevel, SubscribeRequestFilterAccounts, SubscribeRequestFilterTransactions,
};

use crate::raydium_intruction_processor::RaydiumAmmV4InstructionProcessor;

pub const RAYDIUM_AMM_V4_PROGRAM_ID: Pubkey =
    pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");

pub fn make_raydium_geyser_instruction_pipeline() -> Result<Pipeline> {
    // Set up transaction filters to only process Raydium transactions
    let mut transaction_filters = HashMap::new();
    transaction_filters.insert(
        "raydium_transaction_filter".to_string(),
        SubscribeRequestFilterTransactions {
            vote: Some(false),
            failed: Some(false),
            account_include: vec![],
            account_exclude: vec![],
            account_required: vec![RAYDIUM_AMM_V4_PROGRAM_ID.to_string()],
            signature: None,
        },
    );

    // Create empty account filters since we only care about transactions
    let account_filters: HashMap<String, SubscribeRequestFilterAccounts> = HashMap::new();

    let pipeline = Pipeline::builder()
        .datasource(YellowstoneGrpcGeyserClient::new(
            std::env::var("GEYSER_URL").expect("GEYSER_URL is not set"),
            Some(std::env::var("GEYSER_X_TOKEN").expect("GEYSER_X_TOKEN is not set")),
            Some(CommitmentLevel::Processed),
            account_filters,
            transaction_filters,
            Arc::new(RwLock::new(HashSet::new())),
        ))
        .metrics(Arc::new(LogMetrics::new()))
        .instruction(RaydiumAmmV4Decoder, RaydiumAmmV4InstructionProcessor::new())
        .build()?;

    Ok(pipeline)
}
