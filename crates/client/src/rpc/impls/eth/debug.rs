use crate::rpc::traits::eth_space::debug::Debug;
use alloy_rpc_types_trace::geth::{
    GethDebugBuiltInTracerType,
    GethDebugTracerType::{BuiltInTracer, JsTracer},
    GethDebugTracingOptions, GethTrace, NoopFrame,
};
use cfx_types::H256;
use cfxcore::{ConsensusGraph, SharedConsensusGraph};
use jsonrpc_core::{Error as RpcError, Result as JsonRpcResult};
// use primitives::EpochNumber;

pub struct GethDebugHandler {
    consensus: SharedConsensusGraph,
}

impl GethDebugHandler {
    pub fn new(consensus: SharedConsensusGraph) -> Self {
        GethDebugHandler { consensus }
    }

    fn consensus_graph(&self) -> &ConsensusGraph {
        self.consensus
            .as_any()
            .downcast_ref::<ConsensusGraph>()
            .expect("downcast should succeed")
    }
}

impl Debug for GethDebugHandler {
    fn db_get(&self, _key: String) -> JsonRpcResult<Option<String>> {
        Ok(Some("To be implemented!".into()))
    }

    fn debug_trace_transaction(
        &self, hash: H256, opts: Option<GethDebugTracingOptions>,
    ) -> JsonRpcResult<GethTrace> {
        let opts = opts.unwrap_or_default();

        // early return if tracer is not supported or NoopTracer is requested
        if let Some(tracer_type) = &opts.tracer {
            match tracer_type {
                BuiltInTracer(builtin_tracer) => match builtin_tracer {
                    GethDebugBuiltInTracerType::FourByteTracer => {
                        return Err(RpcError::invalid_params("not supported"))
                    }
                    GethDebugBuiltInTracerType::CallTracer => (),
                    GethDebugBuiltInTracerType::PreStateTracer => {
                        return Err(RpcError::invalid_params("not supported"))
                    }
                    GethDebugBuiltInTracerType::NoopTracer => {
                        return Ok(GethTrace::NoopTracer(NoopFrame::default()))
                    }
                    GethDebugBuiltInTracerType::MuxTracer => {
                        return Err(RpcError::invalid_params("not supported"))
                    }
                },
                JsTracer(_) => {
                    return Err(RpcError::invalid_params("not supported"))
                }
            }
        }

        let tx_index = self
            .consensus
            .get_data_manager()
            .transaction_index_by_hash(&hash, false /* update_cache */)
            .ok_or(RpcError::invalid_params("invalid tx hash"))?;

        let epoch_num = self
            .consensus
            .get_block_epoch_number(&tx_index.block_hash)
            .ok_or(RpcError::invalid_params("invalid tx hash"))?;

        let epoch_traces = self
            .consensus_graph()
            .collect_epoch_geth_trace(epoch_num, Some(hash), opts)
            .map_err(|_e| RpcError::invalid_params("invalid tx hash"))?;

        // filter by tx hash
        let trace = epoch_traces
            .into_iter()
            .find(|(tx_hash, _)| tx_hash == &hash)
            .map(|(_, trace)| trace)
            .ok_or(RpcError::invalid_params("invalid tx hash"));

        trace
    }
}
