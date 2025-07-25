use tokio::sync::{broadcast, mpsc, watch};

use crate::{controller::{ControllerCommand, CueController, ShowState}, engine::audio_engine::{AudioCommand, AudioEngine}, event::UiEvent, executor::{EngineEvent, Executor, ExecutorCommand, ExecutorEvent}, manager::ShowModelManager};

mod event;
mod controller;
mod engine;
mod executor;
mod manager;
mod model;

pub struct BackendHandle {
    pub model_manager: ShowModelManager,

    pub controller_tx: mpsc::Sender<ControllerCommand>,
    pub state_rx: watch::Receiver<ShowState>,
    pub event_rx: broadcast::Receiver<UiEvent>
}

pub async fn start_backend() -> BackendHandle {
    let (controller_tx, controller_rx) = mpsc::channel::<ControllerCommand>(32);
    let (exec_tx, exec_rx) = mpsc::channel::<ExecutorCommand>(32);
    let (audio_tx, audio_rx) = mpsc::channel::<AudioCommand>(32);
    let (executor_event_tx, executor_event_rx) = mpsc::channel::<ExecutorEvent>(32);
    let (engine_event_tx, engine_event_rx) = mpsc::channel::<EngineEvent>(32);
    let (state_tx, state_rx) = watch::channel::<ShowState>(ShowState::new());
    let (event_tx, event_rx) = broadcast::channel::<UiEvent>(32);

    let model_manager = ShowModelManager::new();
    let controller = CueController::new(
        model_manager.clone(),
        exec_tx,
        controller_rx,
        executor_event_rx,
        state_tx,
        event_tx.clone(),
    ).await;

    let executor = Executor::new(
        model_manager.clone(),
        exec_rx,
        audio_tx,
        executor_event_tx,
        engine_event_rx,
    );

    let audio_engine = AudioEngine::new(audio_rx, engine_event_tx).unwrap();

    tokio::spawn(controller.run());
    tokio::spawn(executor.run());
    tokio::spawn(audio_engine.run());

    BackendHandle { model_manager, controller_tx, state_rx, event_rx }
}