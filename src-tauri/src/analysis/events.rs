use extractum_analysis::{AnalysisChatEvent, AnalysisEventSink, AnalysisRunEvent};
use tauri::{AppHandle, Emitter};

use super::{ANALYSIS_CHAT_EVENT, ANALYSIS_RUN_EVENT};

pub(super) struct TauriAnalysisEventSink {
    handle: AppHandle,
}

impl TauriAnalysisEventSink {
    pub(super) fn new(handle: AppHandle) -> Self {
        Self { handle }
    }
}

impl AnalysisEventSink for TauriAnalysisEventSink {
    fn publish_run(&self, event: AnalysisRunEvent) {
        let _ = self.handle.emit(ANALYSIS_RUN_EVENT, &event);
    }

    fn publish_chat(&self, event: AnalysisChatEvent) {
        let _ = self.handle.emit(ANALYSIS_CHAT_EVENT, &event);
    }
}
