//! Tauri IPC commands for expansion engine control.

use tauri::State;
use serde::{Serialize, Deserialize};

use super::error::CommandError;
use crate::managers::engine_manager::{EngineManager, EngineStatus};

/// Shared state for the expansion engine.
pub struct EngineState {
    pub engine: std::sync::Mutex<EngineManager>,
}

/// Status response for the engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineStatusResponse {
    pub status: String,
    pub is_running: bool,
    pub is_paused: bool,
}

impl From<EngineStatus> for EngineStatusResponse {
    fn from(status: EngineStatus) -> Self {
        match status {
            EngineStatus::Stopped => Self {
                status: "stopped".to_string(),
                is_running: false,
                is_paused: false,
            },
            EngineStatus::Running => Self {
                status: "running".to_string(),
                is_running: true,
                is_paused: false,
            },
            EngineStatus::Paused => Self {
                status: "paused".to_string(),
                is_running: true,
                is_paused: true,
            },
        }
    }
}

/// Starts the expansion engine.
#[tauri::command]
pub fn start_engine(state: State<EngineState>) -> Result<EngineStatusResponse, CommandError> {
    let engine = state.engine.lock().map_err(|_| CommandError {
        code: "LOCK_ERROR".to_string(),
        message: "Failed to acquire engine lock".to_string(),
    })?;

    engine.start().map_err(|e| CommandError {
        code: "ENGINE_ERROR".to_string(),
        message: format!("Failed to start engine: {}", e),
    })?;

    let status = engine.status().map_err(|e| CommandError {
        code: "ENGINE_ERROR".to_string(),
        message: format!("Failed to get engine status: {}", e),
    })?;

    Ok(status.into())
}

/// Stops the expansion engine.
#[tauri::command]
pub fn stop_engine(state: State<EngineState>) -> Result<EngineStatusResponse, CommandError> {
    let engine = state.engine.lock().map_err(|_| CommandError {
        code: "LOCK_ERROR".to_string(),
        message: "Failed to acquire engine lock".to_string(),
    })?;

    engine.stop().map_err(|e| CommandError {
        code: "ENGINE_ERROR".to_string(),
        message: format!("Failed to stop engine: {}", e),
    })?;

    let status = engine.status().map_err(|e| CommandError {
        code: "ENGINE_ERROR".to_string(),
        message: format!("Failed to get engine status: {}", e),
    })?;

    Ok(status.into())
}

/// Restarts the expansion engine.
#[tauri::command]
pub fn restart_engine(state: State<EngineState>) -> Result<EngineStatusResponse, CommandError> {
    let engine = state.engine.lock().map_err(|_| CommandError {
        code: "LOCK_ERROR".to_string(),
        message: "Failed to acquire engine lock".to_string(),
    })?;

    engine.restart().map_err(|e| CommandError {
        code: "ENGINE_ERROR".to_string(),
        message: format!("Failed to restart engine: {}", e),
    })?;

    let status = engine.status().map_err(|e| CommandError {
        code: "ENGINE_ERROR".to_string(),
        message: format!("Failed to get engine status: {}", e),
    })?;

    Ok(status.into())
}

/// Pauses the expansion engine.
#[tauri::command]
pub fn pause_engine(state: State<EngineState>) -> Result<EngineStatusResponse, CommandError> {
    let engine = state.engine.lock().map_err(|_| CommandError {
        code: "LOCK_ERROR".to_string(),
        message: "Failed to acquire engine lock".to_string(),
    })?;

    engine.pause().map_err(|e| CommandError {
        code: "ENGINE_ERROR".to_string(),
        message: format!("Failed to pause engine: {}", e),
    })?;

    let status = engine.status().map_err(|e| CommandError {
        code: "ENGINE_ERROR".to_string(),
        message: format!("Failed to get engine status: {}", e),
    })?;

    Ok(status.into())
}

/// Resumes the expansion engine.
#[tauri::command]
pub fn resume_engine(state: State<EngineState>) -> Result<EngineStatusResponse, CommandError> {
    let engine = state.engine.lock().map_err(|_| CommandError {
        code: "LOCK_ERROR".to_string(),
        message: "Failed to acquire engine lock".to_string(),
    })?;

    engine.resume().map_err(|e| CommandError {
        code: "ENGINE_ERROR".to_string(),
        message: format!("Failed to resume engine: {}", e),
    })?;

    let status = engine.status().map_err(|e| CommandError {
        code: "ENGINE_ERROR".to_string(),
        message: format!("Failed to get engine status: {}", e),
    })?;

    Ok(status.into())
}

/// Gets the current engine status.
#[tauri::command]
pub fn get_engine_status(state: State<EngineState>) -> Result<EngineStatusResponse, CommandError> {
    let engine = state.engine.lock().map_err(|_| CommandError {
        code: "LOCK_ERROR".to_string(),
        message: "Failed to acquire engine lock".to_string(),
    })?;

    let status = engine.status().map_err(|e| CommandError {
        code: "ENGINE_ERROR".to_string(),
        message: format!("Failed to get engine status: {}", e),
    })?;

    Ok(status.into())
}
