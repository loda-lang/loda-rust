use super::AnalyticsWorkerMessage;
use super::ExecuteBatchResult;
use super::MineEventDirectoryState;
use super::MinerWorkerMessage;
use super::PostmineWorkerMessage;
use bastion::prelude::*;
use std::time::{Duration, Instant};

const RECEIVE_TIMEOUT_SECONDS: u64 = 1; // 1 second
const STOP_MINING_SHUTDOWN_PERIOD_MILLIS: u128 = 2100; // 2.1 seconds

#[derive(Clone, Debug)]
pub enum CoordinatorWorkerMessage {
    RunLaunchProcedure,
    SyncAndAnalyticsIsComplete,
    PostmineJobComplete,

    /// Invoked by the cronjob when it's time for a `sync`.
    /// This synchronizes the `loda-programs` repository.
    /// And regenerates the `~/.loda-rust/analytics` directory.
    TriggerSync,
}

#[derive(Debug, Clone)]
pub enum CoordinatorWorkerQuestion {
    MinerWorkerExecutedOneBatch { execute_batch_result: ExecuteBatchResult },
}

pub async fn coordinator_worker(
    ctx: BastionContext,
) -> Result<(), ()> {
    let timeout = Duration::from_secs(RECEIVE_TIMEOUT_SECONDS);
    let mut state_machine = StateMachine::new();
    loop {
        let message: SignedMessage = match ctx.try_recv_timeout(timeout).await {
            Ok(message) => message,
            Err(error) => {
                if let ReceiveError::Timeout(_duration) = error {
                    state_machine.timeout();
                    continue;
                }
                error!("coordinator_worker: Unknown error happened. error: {:?}", error);
                continue;
            }
        };
        MessageHandler::new(message)
            .on_tell(|message: CoordinatorWorkerMessage, _| {
                debug!(
                    "coordinator_worker: child {}, received message: {:?}",
                    ctx.current().id(),
                    message
                );
                match message {
                    CoordinatorWorkerMessage::RunLaunchProcedure => {
                        state_machine.run_launch_procedure();
                    },
                    CoordinatorWorkerMessage::TriggerSync => {
                        state_machine.trigger_sync();
                    },
                    CoordinatorWorkerMessage::SyncAndAnalyticsIsComplete => {
                        state_machine.sync_and_analytics_is_complete();
                    },
                    CoordinatorWorkerMessage::PostmineJobComplete => {
                        state_machine.postmine_job_is_complete();
                    }
                }
            })
            .on_question(|message: CoordinatorWorkerQuestion, sender| {
                // debug!("coordinator_worker {}, received a question: \n{:?}", 
                //     ctx.current().id(),
                //     message
                // );
                match message {
                    CoordinatorWorkerQuestion::MinerWorkerExecutedOneBatch { execute_batch_result } => {
                        state_machine.miner_worker_executed_one_batch(&execute_batch_result);
                        let reply: String;
                        if state_machine.should_continue_mining() {
                            reply = "continue".to_string();
                        } else {
                            reply = "stop".to_string();
                        }
                        match sender.reply(reply) {
                            Ok(value) => {
                                debug!("coordinator_worker: reply ok: {:?}", value);
                            },
                            Err(error) => {
                                error!("coordinator_worker: reply error: {:?}", error);
                            }
                        };
                    },
                }
            })
            .on_fallback(|unknown, _sender_addr| {
                error!(
                    "coordinator_worker {}, received an unknown message!:\n{:?}",
                    ctx.current().id(),
                    unknown
                );
            });
    }
}

/// tell the `analytics_worker` instance to perform the launch procedure
fn run_launch_procedure() {
    let distributor = Distributor::named("analytics_worker");
    let tell_result = distributor.tell_everyone(AnalyticsWorkerMessage::RunLaunchProcedure);
    if let Err(error) = tell_result {
        Bastion::stop();
        panic!("coordinator_worker: Unable to send RunLaunchProcedure to analytics_worker_distributor. error: {:?}", error);
    }
}

/// tell all `miner_worker` instances to start mining
fn start_mining() {
    let distributor = Distributor::named("miner_worker");
    let tell_result = distributor.tell_everyone(MinerWorkerMessage::StartMining);
    if let Err(error) = tell_result {
        Bastion::stop();
        panic!("coordinator_worker: Unable to send StartMining to miner_worker_distributor. error: {:?}", error);
    }
}

trait State: Send {
    fn run_launch_procedure(self: Box<Self>) -> Box<dyn State>;
    fn sync_and_analytics_is_complete(self: Box<Self>) -> Box<dyn State>;

    fn miner_worker_executed_one_batch(
        self: Box<Self>, 
        execute_batch_result: &ExecuteBatchResult,
    ) -> Box<dyn State>;

    fn should_continue_mining(&self) -> bool;
    fn postmine_job_is_complete(self: Box<Self>) -> Box<dyn State>;
    fn timeout(self: Box<Self>) -> Box<dyn State>;
    fn trigger_sync(self: Box<Self>) -> Box<dyn State>;
}

struct InitialState;

impl State for InitialState {
    fn run_launch_procedure(self: Box<Self>) -> Box<dyn State> {
        run_launch_procedure();
        Box::new(RunLaunchProcedureInProgressState { 
            trigger_sync: false,
            mineevent_dir_state: MineEventDirectoryState::new(),
        })
    }

    fn sync_and_analytics_is_complete(self: Box<Self>) -> Box<dyn State> {
        error!("InitialState.sync_and_analytics_is_complete() called, but is never supposed to be invoked in this state");
        self
    }

    fn miner_worker_executed_one_batch(
        self: Box<Self>, 
        _execute_batch_result: &ExecuteBatchResult,
    ) -> Box<dyn State> {
        error!("InitialState.miner_worker_executed_one_batch() called, but is never supposed to be invoked in this state");
        self
    }

    fn should_continue_mining(&self) -> bool {
        error!("InitialState.should_continue_mining() called, but is never supposed to be invoked in this state");
        false
    }

    fn postmine_job_is_complete(self: Box<Self>) -> Box<dyn State> {
        error!("InitialState.postmine_job_is_complete() called, but is never supposed to be invoked in this state");
        self
    }

    fn timeout(self: Box<Self>) -> Box<dyn State> {
        self
    }

    fn trigger_sync(self: Box<Self>) -> Box<dyn State> {
        self
    }
}

struct RunLaunchProcedureInProgressState {
    trigger_sync: bool,
    mineevent_dir_state: MineEventDirectoryState,
}

impl State for RunLaunchProcedureInProgressState {
    fn run_launch_procedure(self: Box<Self>) -> Box<dyn State> {
        error!("RunLaunchProcedureInProgressState.run_launch_procedure() called, but is never supposed to be invoked in this state");
        self
    }

    fn sync_and_analytics_is_complete(self: Box<Self>) -> Box<dyn State> {
        start_mining();
        Box::new(MiningInProgressState { 
            trigger_sync: self.trigger_sync,
            mineevent_dir_state: self.mineevent_dir_state,
        })
    }

    fn miner_worker_executed_one_batch(
        self: Box<Self>, 
        _execute_batch_result: &ExecuteBatchResult,
    ) -> Box<dyn State> {
        error!("RunLaunchProcedureInProgressState.miner_worker_executed_one_batch() called, but is never supposed to be invoked in this state");
        self
    }

    fn should_continue_mining(&self) -> bool {
        error!("RunLaunchProcedureInProgressState.should_continue_mining() called, but is never supposed to be invoked in this state");
        false
    }

    fn postmine_job_is_complete(self: Box<Self>) -> Box<dyn State> {
        error!("RunLaunchProcedureInProgressState.postmine_job_is_complete() called, but is never supposed to be invoked in this state");
        self
    }

    fn timeout(self: Box<Self>) -> Box<dyn State> {
        self
    }

    fn trigger_sync(self: Box<Self>) -> Box<dyn State> {
        Box::new(Self { 
            trigger_sync: true,
            mineevent_dir_state: self.mineevent_dir_state,
        })
    }
}

struct MiningInProgressState {
    trigger_sync: bool,
    mineevent_dir_state: MineEventDirectoryState,
}

impl State for MiningInProgressState {
    fn run_launch_procedure(self: Box<Self>) -> Box<dyn State> {
        error!("MiningInProgressState.run_launch_procedure() called, but is never supposed to be invoked in this state");
        self
    }

    fn sync_and_analytics_is_complete(self: Box<Self>) -> Box<dyn State> {
        error!("MiningInProgressState.sync_and_analytics_is_complete() called, but is never supposed to be invoked in this state");
        self
    }

    fn miner_worker_executed_one_batch(
        self: Box<Self>, 
        execute_batch_result: &ExecuteBatchResult,
    ) -> Box<dyn State> {
        debug!("MiningInProgressState: executed one batch: {:?}", execute_batch_result);
        let mut mineevent_dir_state = self.mineevent_dir_state.clone();
        mineevent_dir_state.accumulate_stats(&execute_batch_result);
        if !mineevent_dir_state.has_reached_mining_limit() {
            // Stay in this state, and accumulate candidate programs
            return Box::new(MiningInProgressState {
                trigger_sync: self.trigger_sync, 
                mineevent_dir_state
            });
        }
        debug!("MiningInProgressState: the number of accumulated candiate programs has reached the limit");
        Box::new(MiningIsStoppingState::new(
            self.trigger_sync, 
            mineevent_dir_state
        ))
    }

    fn should_continue_mining(&self) -> bool {
        // Keep the miner_workers busy with mining
        true
    }

    fn postmine_job_is_complete(self: Box<Self>) -> Box<dyn State> {
        error!("MiningInProgressState.postmine_job_is_complete() called, but is never supposed to be invoked in this state");
        self
    }

    fn timeout(self: Box<Self>) -> Box<dyn State> {
        self
    }

    fn trigger_sync(self: Box<Self>) -> Box<dyn State> {
        debug!("MiningInProgressState: Stop mining immediately, so the cronjob can be performed as soon as possible");
        Box::new(MiningIsStoppingState::new(
            true, 
            self.mineevent_dir_state,
        ))
    }
}

/// Wait for all miner_worker instances to complete their mining job
struct MiningIsStoppingState {
    trigger_sync: bool,
    mineevent_dir_state: MineEventDirectoryState,
    start_time: Instant,
}

impl MiningIsStoppingState {
    fn new(trigger_sync: bool, mineevent_dir_state: MineEventDirectoryState) -> Self {
        let start_time = Instant::now();
        Self { 
            trigger_sync,
            mineevent_dir_state,
            start_time 
        }
    }
}

impl State for MiningIsStoppingState {
    fn run_launch_procedure(self: Box<Self>) -> Box<dyn State> {
        error!("MiningIsStoppingState.run_launch_procedure() called, but is never supposed to be invoked in this state");
        self
    }

    fn sync_and_analytics_is_complete(self: Box<Self>) -> Box<dyn State> {
        error!("MiningIsStoppingState.sync_and_analytics_is_complete() called, but is never supposed to be invoked in this state");
        self
    }

    fn miner_worker_executed_one_batch(
        self: Box<Self>, 
        execute_batch_result: &ExecuteBatchResult,
    ) -> Box<dyn State> {
        debug!("MiningIsStoppingState: executed one batch: {:?}", execute_batch_result);
        let mut mineevent_dir_state = self.mineevent_dir_state.clone();
        mineevent_dir_state.accumulate_stats(&execute_batch_result);
        Box::new(MiningIsStoppingState::new(
            self.trigger_sync, 
            mineevent_dir_state
        ))
    }

    fn should_continue_mining(&self) -> bool {
        // Tell miner_worker to stop mining
        false
    }

    fn postmine_job_is_complete(self: Box<Self>) -> Box<dyn State> {
        error!("MiningIsStoppingState.postmine_job_is_complete() called, but is never supposed to be invoked in this state");
        self
    }

    fn timeout(self: Box<Self>) -> Box<dyn State> {
        let elapsed: u128 = self.start_time.elapsed().as_millis();
        if elapsed < STOP_MINING_SHUTDOWN_PERIOD_MILLIS {
            // Stay in this state while waiting for all the miner_worker instances 
            // to complete their mining jobs
            return self;
        }

        println!("MiningIsStoppingState: trigger start postmine");
        let distributor = Distributor::named("postmine_worker");
        let tell_result = distributor
            .tell_everyone(PostmineWorkerMessage::StartPostmineJob);
        if let Err(error) = tell_result {
            error!("MiningIsStoppingState: Unable to send StartPostmineJob. error: {:?}", error);
        }
        
        // Transition to the "postmine" state.
        return Box::new(PostmineInProgressState { 
            trigger_sync: self.trigger_sync,
            mineevent_dir_state: self.mineevent_dir_state,
        });
    }

    fn trigger_sync(self: Box<Self>) -> Box<dyn State> {
        Box::new(Self { 
            trigger_sync: true,
            mineevent_dir_state: self.mineevent_dir_state,
            start_time: self.start_time, 
        })
    }
}

struct PostmineInProgressState {
    trigger_sync: bool,
    mineevent_dir_state: MineEventDirectoryState,
}

impl State for PostmineInProgressState {
    fn run_launch_procedure(self: Box<Self>) -> Box<dyn State> {
        error!("PostmineInProgressState.run_launch_procedure() called, but is never supposed to be invoked in this state");
        self
    }

    fn sync_and_analytics_is_complete(self: Box<Self>) -> Box<dyn State> {
        error!("PostmineInProgressState.sync_and_analytics_is_complete() called, but is never supposed to be invoked in this state");
        self
    }

    fn miner_worker_executed_one_batch(
        self: Box<Self>, 
        _execute_batch_result: &ExecuteBatchResult,
    ) -> Box<dyn State> {
        error!("PostmineInProgressState.miner_worker_executed_one_batch() called, but is never supposed to be invoked in this state");
        self
    }

    fn should_continue_mining(&self) -> bool {
        error!("PostmineInProgressState.should_continue_mining() called, but is never supposed to be invoked in this state");
        false
    }

    fn postmine_job_is_complete(self: Box<Self>) -> Box<dyn State> {
        if self.trigger_sync {
            println!("PostmineInProgressState: postmine job is complete. perform the scheduled \"sync\" task");
            run_launch_procedure();
            return Box::new(RunLaunchProcedureInProgressState { 
                trigger_sync: false, // Clear the trigger_sync flag, since we have just performed it.
                mineevent_dir_state: MineEventDirectoryState::new(), // Reset the mine-event directory counters
            });
        }

        println!("PostmineInProgressState: postmine job is complete. Resume mining again");
        start_mining();
        Box::new(MiningInProgressState {
            trigger_sync: false, // Clear the trigger_sync flag, since we have just performed it.
            mineevent_dir_state: MineEventDirectoryState::new(), // Reset the mine-event directory counters
        })
    }

    fn timeout(self: Box<Self>) -> Box<dyn State> {
        self
    }

    fn trigger_sync(self: Box<Self>) -> Box<dyn State> {
        Box::new(Self { 
            trigger_sync: true,
            mineevent_dir_state: self.mineevent_dir_state,
        })
    }
}

struct StateMachine {
    state: Option<Box<dyn State>>,
}

impl StateMachine {
    fn new() -> Self {
        Self {
            state: Some(Box::new(InitialState {})),
        }
    }

    fn run_launch_procedure(&mut self) {
        if let Some(state) = self.state.take() {
            self.state = Some(state.run_launch_procedure());
        }
    }

    fn sync_and_analytics_is_complete(&mut self) {
        if let Some(state) = self.state.take() {
            self.state = Some(state.sync_and_analytics_is_complete());
        }
    }

    fn miner_worker_executed_one_batch(&mut self, execute_batch_result: &ExecuteBatchResult) {
        if let Some(state) = self.state.take() {
            let new_state = state.miner_worker_executed_one_batch(
                execute_batch_result
            );
            self.state = Some(new_state);
        }
    }

    fn should_continue_mining(&mut self) -> bool {
        if let Some(state) = &self.state {
            return state.should_continue_mining();
        }
        false
    }

    fn postmine_job_is_complete(&mut self) {
        if let Some(state) = self.state.take() {
            self.state = Some(state.postmine_job_is_complete());
        }
    }

    fn timeout(&mut self) {
        if let Some(state) = self.state.take() {
            self.state = Some(state.timeout());
        }
    }

    fn trigger_sync(&mut self) {
        println!("coordinator_worker: trigger_sync - scheduling \"sync\" as soon as possible");
        if let Some(state) = self.state.take() {
            self.state = Some(state.trigger_sync());
        }
    }
}
