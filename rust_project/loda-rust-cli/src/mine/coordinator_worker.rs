use super::AnalyticsWorkerMessage;
use super::ExecuteBatchResult;
use super::MineEventDirectoryState;
use super::MinerWorkerMessage;
use super::PostmineWorkerMessage;
use bastion::prelude::*;
use std::time::{Duration, Instant};

const RECEIVE_TIMEOUT_SECONDS: u64 = 1; // 1 second
const STOP_MINING_SHUTDOWN_PERIOD_MILLIS: u128 = 1500; // 1.5 second

#[derive(Clone, Debug)]
pub enum CoordinatorWorkerMessage {
    RunLaunchProcedure,
    SyncAndAnalyticsIsComplete,
    PostmineJobComplete,
    CronjobTriggerSync,
}

#[derive(Debug, Clone)]
pub enum CoordinatorWorkerQuestion {
    MinerWorkerExecutedOneBatch { execute_batch_result: ExecuteBatchResult },
}

pub async fn coordinator_worker(
    ctx: BastionContext,
) -> Result<(), ()> {
    let timeout = Duration::from_secs(RECEIVE_TIMEOUT_SECONDS);
    let mut mineevent_dir_state = MineEventDirectoryState::new();
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
                    CoordinatorWorkerMessage::CronjobTriggerSync => {
                        println!("!!!!!!!!! trigger sync")
                    },
                    CoordinatorWorkerMessage::SyncAndAnalyticsIsComplete => {
                        state_machine.sync_and_analytics_is_complete();
                    },
                    CoordinatorWorkerMessage::PostmineJobComplete => {
                        state_machine.postmine_job_is_complete();
                        mineevent_dir_state.reset();
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
                        state_machine.miner_worker_executed_one_batch(&execute_batch_result, &mut mineevent_dir_state);
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
        mineevent_directory_state: &mut MineEventDirectoryState,
    ) -> Box<dyn State>;

    fn should_continue_mining(&self) -> bool;

    fn postmine_job_is_complete(self: Box<Self>) -> Box<dyn State>;

    fn timeout(self: Box<Self>) -> Box<dyn State>;
}

struct InitialState;

impl State for InitialState {
    fn run_launch_procedure(self: Box<Self>) -> Box<dyn State> {
        run_launch_procedure();
        Box::new(RunLaunchProcedureInProgressState {})
    }

    fn sync_and_analytics_is_complete(self: Box<Self>) -> Box<dyn State> {
        error!("InitialState.sync_and_analytics_is_complete() called, but is never supposed to be invoked in this state");
        self
    }

    fn miner_worker_executed_one_batch(
        self: Box<Self>, 
        _execute_batch_result: &ExecuteBatchResult,
        _mineevent_directory_state: &mut MineEventDirectoryState,
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
}

struct RunLaunchProcedureInProgressState;

impl State for RunLaunchProcedureInProgressState {
    fn run_launch_procedure(self: Box<Self>) -> Box<dyn State> {
        error!("RunLaunchProcedureInProgressState.run_launch_procedure() called, but is never supposed to be invoked in this state");
        self
    }

    fn sync_and_analytics_is_complete(self: Box<Self>) -> Box<dyn State> {
        start_mining();
        Box::new(MiningInProgressState {})
    }

    fn miner_worker_executed_one_batch(
        self: Box<Self>, 
        _execute_batch_result: &ExecuteBatchResult,
        _mineevent_directory_state: &mut MineEventDirectoryState,
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
}

struct MiningInProgressState;

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
        mineevent_directory_state: &mut MineEventDirectoryState,
    ) -> Box<dyn State> {
        debug!("MiningInProgressState: executed one batch: {:?}", execute_batch_result);
        mineevent_directory_state.accumulate_stats(&execute_batch_result);

        if !mineevent_directory_state.has_reached_mining_limit() {
            // Stay in this state, and accumulate candidate programs
            return self;
        }
        println!("MiningInProgressState. the number of accumulated candiate programs has reached the limit");
        return Box::new(MiningIsStoppingState::new());
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
}

/// Wait for all miner_worker instances to complete their mining job
struct MiningIsStoppingState {
    start_time: Instant,
}

impl MiningIsStoppingState {
    fn new() -> Self {
        let start_time = Instant::now();
        Self { start_time }
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
        mineevent_directory_state: &mut MineEventDirectoryState,
    ) -> Box<dyn State> {
        debug!("MiningIsStoppingState: executed one batch: {:?}", execute_batch_result);
        mineevent_directory_state.accumulate_stats(&execute_batch_result);
        return self;
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
        return Box::new(PostmineInProgressState {});
    }
}

struct PostmineInProgressState;

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
        _mineevent_directory_state: &mut MineEventDirectoryState,
    ) -> Box<dyn State> {
        error!("PostmineInProgressState.miner_worker_executed_one_batch() called, but is never supposed to be invoked in this state");
        self
    }

    fn should_continue_mining(&self) -> bool {
        error!("PostmineInProgressState.should_continue_mining() called, but is never supposed to be invoked in this state");
        false
    }

    fn postmine_job_is_complete(self: Box<Self>) -> Box<dyn State> {
        println!("PostmineInProgressState: postmine job is complete. Resume mining again");
        start_mining();
        Box::new(MiningInProgressState {})
    }

    fn timeout(self: Box<Self>) -> Box<dyn State> {
        self
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

    fn miner_worker_executed_one_batch(&mut self, execute_batch_result: &ExecuteBatchResult, mineevent_directory_state: &mut MineEventDirectoryState) {
        if let Some(state) = self.state.take() {
            let new_state = state.miner_worker_executed_one_batch(
                execute_batch_result, 
                mineevent_directory_state
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
}
