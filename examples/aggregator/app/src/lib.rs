#![no_std]

use demo_client::{DemoClient, DemoClientProgram, chaos::Chaos, counter::Counter};
use futures::{FutureExt, future};
use msg_tracker::{
    MsgTracker, OpStatus, TrackerBackend, TrackerBenchResult, TrackerOp, message_id_for_seed,
};
use redirect_client::{redirect::Redirect as _, *};
use sails_rs::{cell::RefCell, client::*, gstd::msg, prelude::*};

pub mod msg_tracker;

static mut MSG_TRACKER: Option<RefCell<MsgTracker>> = None;

pub fn msg_tracker() -> &'static RefCell<MsgTracker> {
    unsafe {
        (*core::ptr::addr_of_mut!(MSG_TRACKER))
            .as_ref()
            .unwrap_or_else(|| panic!("`MsgTracker` data should be initialized first"))
    }
}

pub struct AggregatorService {
    counter: Service<demo_client::counter::CounterImpl>,
}

impl AggregatorService {
    pub fn new(target: ActorId) -> Self {
        let demo = DemoClientProgram::client(target);
        Self {
            counter: demo.counter(),
        }
    }
}

#[sails_rs::service]
impl AggregatorService {
    #[export]
    pub async fn fetch_value(&self) -> Result<u32, String> {
        self.counter
            .value()
            .send_for_reply()
            .map_err(|e| e.to_string())?
            .await
            .map_err(|e| e.to_string())
    }

    #[export]
    pub async fn fetch_summary(&self) -> Result<(u32, u32), String> {
        let call1 = self
            .counter
            .value()
            .send_for_reply()
            .map_err(|e| e.to_string())?;
        let call2 = self
            .counter
            .value()
            .send_for_reply()
            .map_err(|e| e.to_string())?;
        let (res1, res2) = future::join(call1, call2).await;
        Ok((
            res1.map_err(|e| e.to_string())?,
            res2.map_err(|e| e.to_string())?,
        ))
    }

    #[export]
    pub async fn fetch_with_fallback(&self, use_fallback: bool) -> Result<u32, String> {
        let call = self
            .counter
            .value()
            .send_for_reply()
            .map_err(|e| e.to_string())?;
        if use_fallback {
            let fallback = future::ready(Ok::<u32, sails_rs::errors::Error>(999));
            match future::select(call, fallback.boxed()).await {
                future::Either::Left((res, _)) => res.map_err(|e| e.to_string()),
                future::Either::Right((res, _)) => res.map_err(|e| e.to_string()),
            }
        } else {
            call.await.map_err(|e| e.to_string())
        }
    }

    #[export]
    pub async fn fetch_fastest(&self, target: ActorId) -> Result<u32, String> {
        let demo = DemoClientProgram::client(target);
        let slow_call = demo
            .chaos()
            .timeout_wait()
            .send_for_reply()
            .map_err(|e| e.to_string())?;
        let fast_call = demo
            .counter()
            .value()
            .send_for_reply()
            .map_err(|e| e.to_string())?;
        match future::select(slow_call.fuse(), fast_call.fuse()).await {
            future::Either::Left((res, _)) => {
                res.map_err(|e| e.to_string())?;
                Ok(2)
            }
            future::Either::Right((res, _)) => {
                res.map_err(|e| e.to_string())?;
                Ok(1)
            }
        }
    }

    #[export]
    pub async fn fetch_redirect_id(&self, target: ActorId) -> Result<ActorId, String> {
        let redirect_program = RedirectClientProgram::client(target);
        redirect_program
            .redirect()
            .get_program_id()
            .with_redirect_on_exit(true)
            .send_for_reply()
            .map_err(|e| e.to_string())?
            .await
            .map_err(|e| e.to_string())
    }

    #[export]
    pub async fn test_poll_after_completion(&self) {
        let call = self
            .counter
            .value()
            .send_for_reply()
            .expect("First send failed");
        call.send_for_reply().expect("Second send failed");
    }

    #[export]
    pub async fn fetch_from_address(&self, target: ActorId) -> Result<u32, String> {
        let demo = DemoClientProgram::client(target);
        let call = demo
            .counter()
            .value()
            .send_for_reply()
            .map_err(|e| e.to_string())?;
        call.await.map_err(|e| e.to_string())
    }

    #[export]
    pub fn get_statuses(&self) -> Vec<(MessageId, OpStatus)> {
        msg_tracker().borrow().get_statuses()
    }

    #[export]
    pub fn prepare_tracker(&mut self, len: u32) -> TrackerBenchResult {
        let mut tracker = msg_tracker().borrow_mut();
        tracker.clear();
        for seed in 1..=len {
            tracker.insert(message_id_for_seed(seed), OpStatus::Started);
        }
        TrackerBenchResult {
            len: tracker.len(),
            status: None,
            existed: false,
        }
    }

    #[export]
    pub fn bench_tracker(&mut self, op: TrackerOp, seed: u32) -> TrackerBenchResult {
        let mut tracker = msg_tracker().borrow_mut();
        match op {
            TrackerOp::InsertFresh => {
                let existed = tracker
                    .insert_inner(message_id_for_seed(seed), OpStatus::Started)
                    .is_some();
                TrackerBenchResult {
                    len: tracker.len(),
                    status: Some(OpStatus::Started),
                    existed,
                }
            }
            TrackerOp::UpdateExisting => {
                let existed = tracker.update_status(message_id_for_seed(seed), OpStatus::Finalized);
                TrackerBenchResult {
                    len: tracker.len(),
                    status: Some(OpStatus::Finalized),
                    existed,
                }
            }
            TrackerOp::ReadExisting => {
                let status = tracker.get_status(&message_id_for_seed(seed));
                TrackerBenchResult {
                    len: tracker.len(),
                    status,
                    existed: status.is_some(),
                }
            }
            TrackerOp::ListStatuses => {
                let statuses = tracker.get_statuses();
                TrackerBenchResult {
                    len: statuses.len() as u32,
                    status: statuses.last().map(|(_, status)| *status),
                    existed: !statuses.is_empty(),
                }
            }
        }
    }

    #[export]
    pub async fn complex_add(&mut self) -> Result<u32, String> {
        let parent_id = msg::id();
        msg_tracker()
            .borrow_mut()
            .insert(parent_id, OpStatus::Started);

        let call1 = self
            .counter
            .add(10)
            .with_reply_deposit(10_000_000_000)
            .with_reply_hook(move || {
                msg_tracker()
                    .borrow_mut()
                    .update_status(parent_id, OpStatus::Step1);
            })
            .send_for_reply()
            .map_err(|e| e.to_string())?;

        let call2 = self
            .counter
            .add(20)
            .with_reply_deposit(10_000_000_000)
            .with_reply_hook(move || {
                let mut tracker = msg_tracker().borrow_mut();
                if let Some(OpStatus::Step1) = tracker.get_status(&parent_id) {
                    tracker.update_status(parent_id, OpStatus::Step2);
                }
            })
            .send_for_reply()
            .map_err(|e| e.to_string())?;

        let call3 = self
            .counter
            .add(30)
            .with_wait_up_to(10)
            .send_for_reply()
            .map_err(|e| e.to_string())?;

        let (res1, res2, res3) = future::join3(call1, call2, call3).await;
        res1.map_err(|e| e.to_string())?;
        res2.map_err(|e| e.to_string())?;
        res3.map_err(|e| e.to_string())?;

        msg_tracker()
            .borrow_mut()
            .update_status(parent_id, OpStatus::Finalized);

        // Return final value
        self.counter
            .value()
            .send_for_reply()
            .map_err(|e| e.to_string())?
            .await
            .map_err(|e| e.to_string())
    }
}

pub struct AggregatorProgram {
    target: ActorId,
}

#[sails_rs::program]
impl AggregatorProgram {
    pub fn new(target: ActorId) -> Self {
        Self::new_with_tracker(target, TrackerBackend::BTree)
    }

    pub fn new_with_tracker(target: ActorId, tracker_backend: TrackerBackend) -> Self {
        unsafe {
            MSG_TRACKER = Some(RefCell::new(MsgTracker::new(tracker_backend)));
        }
        Self { target }
    }

    pub fn aggregator(&self) -> AggregatorService {
        AggregatorService::new(self.target)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use code::WASM_BINARY_OPT as WASM_BINARY;

#[cfg(not(target_arch = "wasm32"))]
mod code {
    include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));
}
