use alloy::eips::BlockNumberOrTag;
use alloy::primitives::{utils::keccak256, Address, Bytes, FixedBytes};
use alloy::providers::{ext::DebugApi, Provider, ProviderBuilder, RootProvider};
use alloy::rpc::types::trace::geth::{
    CallFrame, GethDebugBuiltInTracerType, GethDebugTracerType, GethDebugTracingOptions,
    GethDefaultTracingOptions,
};
use alloy::rpc::types::Filter;
use alloy::transports::http::Http;

use reqwest::Client;

#[derive(Clone)]
pub struct Web3Lib {
    pub provider: RootProvider<Http<Client>>,
    pub rpc_url: String,
}

impl Web3Lib {
    pub fn new(rpc_url: String) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Web3Lib {
            rpc_url: rpc_url.clone(),
            provider: ProviderBuilder::new().on_http(rpc_url.parse()?),
        })
    }

    pub async fn get_chain_id(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        // let rpc_url = STATE_MANAGER.app_state.lock()?.rpc_url.clone();

        // if rpc_url.is_none() {
        //     return Err("RPC URL not set".into());
        // }

        // // Create a provider with the HTTP transport using the `reqwest` crate.
        // let provider = ProviderBuilder::new().on_http(rpc_url.unwrap().parse()?);

        let chain_id = self.provider.get_chain_id().await?;
        Ok(chain_id.to_string())
    }

    pub async fn get_block_number_by_transaction_hash(
        &mut self,
        transaction_hash: FixedBytes<32>,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        let block_number = self
            .provider
            .get_transaction_receipt(transaction_hash)
            .await?
            .unwrap()
            .block_number;
        Ok(block_number.unwrap())
    }

    pub async fn get_block_timestamp_by_block_number(
        &mut self,
        block_number: u64,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        let timestamp = self
            .provider
            .get_block_by_number(BlockNumberOrTag::Number(block_number), false)
            .await?
            .unwrap()
            .header
            .timestamp;
        Ok(timestamp)
    }

    pub async fn get_v2_pool_init_code_hash(
        &self,
        v2_factory: Address,
        from_block: u64,
    ) -> Result<FixedBytes<32>, Box<dyn std::error::Error>> {
        let step = 1000;
        let mut i = 0;
        loop {
            let filter: Filter = Filter::new()
                .address(v2_factory)
                .event("PairCreated(address,address,address,uint256)")
                .from_block(from_block + i * step)
                .to_block(from_block + (i + 1) * step);
            let logs = self.provider.get_logs(&filter).await.unwrap();
            if !logs.is_empty() {
                let creation_tx_hash = logs[0].transaction_hash.unwrap();
                let pool_address = Address::from_slice(&logs[0].data().data[12..32]);
                return Ok(self
                    .get_init_code_hash(pool_address, creation_tx_hash)
                    .await?);
            }
            i += 1;
        }
    }

    pub async fn get_v3_pool_init_code_hash(
        &self,
        v3_factory: Address,
        from_block: u64,
    ) -> Result<FixedBytes<32>, Box<dyn std::error::Error>> {
        let step = 1000;
        let mut i = 0;
        loop {
            let filter: Filter = Filter::new()
                .address(v3_factory)
                .event("PoolCreated(address,address,uint24,int24,address)")
                .from_block(from_block + i * step)
                .to_block(from_block + (i + 1) * step);
            let logs = self.provider.get_logs(&filter).await.unwrap();
            if !logs.is_empty() {
                let creation_tx_hash = logs[0].transaction_hash.unwrap();
                let pool_address =
                    Address::from_slice(&logs[0].data().data[logs[0].data().data.len() - 20..]);
                return Ok(self
                    .get_init_code_hash(pool_address, creation_tx_hash)
                    .await?);
            }
            i += 1;
        }
    }

    async fn get_init_code_hash(
        &self,
        factory_address: Address,
        pool_deploy_tx: FixedBytes<32>,
    ) -> Result<FixedBytes<32>, Box<dyn std::error::Error>> {
        let call_options = GethDebugTracingOptions {
            config: GethDefaultTracingOptions {
                disable_storage: Some(true),
                enable_memory: Some(false),
                ..Default::default()
            },
            tracer: Some(GethDebugTracerType::BuiltInTracer(
                GethDebugBuiltInTracerType::CallTracer,
            )),
            ..Default::default()
        };

        let result = self
            .provider
            .debug_trace_transaction(pool_deploy_tx, call_options)
            .await?;

        let calls = result.try_into_call_frame()?.calls;
        let init_code = get_init_code_from_calls(calls, factory_address);
        if init_code.is_none() {
            return Err("No init code found".into());
        }

        let init_code_hash = keccak256(init_code.unwrap());
        Ok(init_code_hash)
    }
}

// recursively search calls for CREATE2 calls and return the init code
fn get_init_code_from_calls(calls: Vec<CallFrame>, contract_address: Address) -> Option<Bytes> {
    for call in calls {
        if call.typ == "CREATE2" && call.to == Some(contract_address) {
            return Some(call.input);
        }
        let init_code = get_init_code_from_calls(call.calls, contract_address);
        if init_code.is_some() {
            return init_code;
        }
    }
    None
}
