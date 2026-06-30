use tauri::{AppHandle, Emitter};

use super::models::{AnalysisChatEvent, AnalysisRunEvent};
use super::{ANALYSIS_CHAT_EVENT, ANALYSIS_RUN_EVENT};

pub(super) fn emit_analysis_event(handle: &AppHandle, event: &AnalysisRunEvent) {
    let _ = handle.emit(ANALYSIS_RUN_EVENT, event);
}

pub(super) fn emit_analysis_chat_event(handle: &AppHandle, event: &AnalysisChatEvent) {
    let _ = handle.emit(ANALYSIS_CHAT_EVENT, event);
}
