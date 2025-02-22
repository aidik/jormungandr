use super::logs::internal::Logs;
use super::pool::internal::Pool;
use crate::{
    blockcfg::{BlockBuilder, HeaderContentEvalContext, Ledger, LedgerParameters},
    fragment::FragmentId,
};
use chain_core::property::Fragment as _;
use jormungandr_lib::interfaces::FragmentStatus;

pub enum SelectionOutput {
    Commit { fragment_id: FragmentId },
    RequestSmallerFee,
    RequestSmallerSize,
    Reject { reason: String },
}

pub trait FragmentSelectionAlgorithm {
    fn select(
        &mut self,
        ledger: &Ledger,
        ledger_params: &LedgerParameters,
        metadata: &HeaderContentEvalContext,
        logs: &mut Logs,
        pool: &mut Pool,
    );

    fn finalize(self) -> BlockBuilder;
}

pub struct OldestFirst {
    builder: BlockBuilder,
    max_per_block: usize,
}

impl OldestFirst {
    pub fn new(max_per_block: usize) -> Self {
        OldestFirst {
            builder: BlockBuilder::new(),
            max_per_block,
        }
    }
}

impl FragmentSelectionAlgorithm for OldestFirst {
    fn finalize(self) -> BlockBuilder {
        self.builder
    }

    fn select(
        &mut self,
        ledger: &Ledger,
        ledger_params: &LedgerParameters,
        metadata: &HeaderContentEvalContext,
        logs: &mut Logs,
        pool: &mut Pool,
    ) {
        let mut total = 0usize;
        let mut ledger_simulation = ledger.clone();

        while let Some(fragment) = pool.remove_oldest() {
            let id = fragment.id();
            match ledger_simulation.apply_fragment(ledger_params, &fragment, metadata) {
                Ok(ledger_new) => {
                    self.builder.message(fragment);

                    total += 1;
                    ledger_simulation = ledger_new;
                }
                Err(error) => {
                    use std::error::Error as _;
                    let error = if let Some(source) = error.source() {
                        format!("{}: {}", error, source)
                    } else {
                        error.to_string()
                    };
                    logs.modify(&id.into(), FragmentStatus::Rejected { reason: error })
                }
            }
            if total >= self.max_per_block {
                break;
            }
        }
    }
}
