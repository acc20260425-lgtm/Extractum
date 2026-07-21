use extractum_core::error::{AppError, AppResult};
use serde::Deserialize;

use super::assets::{SYNTHESIS_RUNTIME_JSON, TRANSCRIPT_RUNTIME_JSON};
use super::json_repair::JsonRepairStageExecutionRequest;
use super::youtube_summary::{
    GemAnalysisPart, GemAnalysisPartRepairRequest, GemAnalysisPartStageExecutionRequest,
    TranscriptAnalysisStageExecutionRequest,
};
use crate::llm::{LlmChatRequest, LlmMessage};

#[derive(Deserialize)]
struct StageRuntimeConfigAsset {
    runtime_configuration: Option<StageRuntimeConfiguration>,
}

#[derive(Deserialize)]
struct StageRuntimeConfiguration {
    budget_limits: Option<StageBudgetLimits>,
}

#[derive(Deserialize)]
struct StageBudgetLimits {
    max_prompt_tokens: Option<i64>,
    max_output_tokens: Option<i64>,
}

pub(super) const DETAILED_REPORT_CONTROL_PRESET: &str = "detailed_report";

const STANDARD_VIDEO_SUMMARY_PROMPT: &str = "Write 2 to 4 paragraphs in the requested output_language, covering the main argument, important context, and practical takeaways. Keep it grounded in the frozen transcript; do not copy long transcript passages.";

const DETAILED_VIDEO_SUMMARY_PROMPT: &str = r#"Put the full Markdown report inside video_candidate.summary_text. This must be the full report, not a short abstract. Minimum length: 800 words when the transcript has enough substance. Keep the response as strict JSON; escape Markdown as a JSON string. Use only the frozen transcript and provided metadata/material refs. Do not claim external verification unless the frozen input contains it.

**Системная роль:**

Вы — ведущий аналитик видеоконтента и эксперт по структурированию знаний. Ваша специализация — деконструкция сложных видео (обучение, лекции, интервью) в атомарные инструкции и глубокие аналитические отчеты.

### 1. Цели и задачи:**

* Предоставлять глубокий технический и смысловой анализ YouTube-видео.
* Создавать структурированные отчеты, включающие метаданные, эссенцию, пошаговые руководства и интерактивные пересказы.
* Использовать внешние ресурсы для проверки фактов и контекста.

---

### 2. Структура ответа

#### I. Метаданные и Контекст

* **Тип контента:** [Обучение / Новости / Интервью / Аналитика]
* **Наличие пошаговых инструкций:** [Да / Нет] (укажите сразу, содержит ли видео четкий алгоритм действий).
* **Целевая аудитория:** Кому и почему это полезно.
* **Инфо-карта:** Название видео (гиперссылка)| Автор (название канала), подписчики| Метрики: [Длительность, Дата, Охват].
* **Таймлайн:** Список ключевых этапов видео с таймкодами.

#### II. Эссенция (Суть)

* **Main Idea:** Главная мысль одним емким предложением.
* **Ключевые тезисы:** 3–5 пунктов с итоговыми выводами (факты, советы, цитаты).
* **Action Plan:** 2-3 конкретных шага: что сделать пользователю сразу после просмотра.

#### III. Пошаговое руководство (How-to) — НОВЫЙ БЛОК

*Этот блок обязателен, если в видео есть процесс (настройка ПО, рецепт, стратегия).*

* **Цель инструкции:** Какой результат получит пользователь.
* **Инструменты:** Что понадобится (сервисы, софт, ингредиенты).
* **Алгоритм:** Детальный нумерованный список. Каждый шаг включает:

1. **Действие:** Что делать.
2. **Таймкод:** `[MM:SS]` как ссылка.
3. **Нюанс:** Важное замечание от автора (чего избегать).

#### IV. Адаптивный модуль (Выполняется в зависимости от типа)

* **Для Обучения:** Глоссарий сложных терминов + Практическое задание для закрепления.
* **Для Новостей:** Список действующих лиц + Исторический/политический контекст (предыстория).
* **Для видео > 20 минут:** Раздел «FAQ: Часто задаваемые вопросы» (5 пар вопрос-ответ на основе видео).

#### V. Глубокий интерактивный пересказ

* **Объем:** Минимум 800-1000 слов. Никакой «воды», только плотный концентрат информации.
* **Структура:** Разбейте на главы с осмысленными заголовками.
* **Навигация:** Каждому важному факту или мысли ОБЯЗАТЕЛЬНО должен сопутствовать таймкод в формате `[ММ:СС]`, являющийся ссылкой.
* **Форматирование:** Используйте таблицы для сравнения характеристик, списки для перечисления.
* **Математика и Код:** Если в видео есть формулы — используйте LaTeX (например, $E=mc^2$). Если код — используйте блоки кода с указанием языка.

---

### 3. Правила оформления и Тон

1. **Язык:** Строго русский.
2. **Стиль:** Профессиональный, аналитический, без «воды».
3. **Визуальный стиль:**

* Заголовки `##` и `###`.
* Разделители `---` между крупными блоками.
* **Жирный шрифт** для ключевых понятий и определений.
* Цитаты `> ` для прямых высказываний автора.

4. **Запрет:** Не использовать фразы «В этом видео говорится...», «Автор рассказывает...». Сразу переходите к сути: «Метод X заключается в...»."#;

pub(super) fn transcript_analysis_control_preset(prompt_input_json: &str) -> String {
    serde_json::from_str::<serde_json::Value>(prompt_input_json)
        .ok()
        .and_then(|input| {
            input
                .get("controlPreset")
                .or_else(|| input.get("control_preset"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| "standard".to_string())
}

fn transcript_analysis_summary_prompt(control_preset: &str) -> &'static str {
    if control_preset == DETAILED_REPORT_CONTROL_PRESET {
        DETAILED_VIDEO_SUMMARY_PROMPT
    } else {
        STANDARD_VIDEO_SUMMARY_PROMPT
    }
}

pub(super) fn build_transcript_analysis_llm_request(
    request: &TranscriptAnalysisStageExecutionRequest,
    profile_id: Option<String>,
    model_override: Option<String>,
    max_output_tokens: Option<i64>,
) -> LlmChatRequest {
    let control_preset = transcript_analysis_control_preset(&request.prompt_input_json);
    let summary_prompt = transcript_analysis_summary_prompt(&control_preset);
    LlmChatRequest {
        request_id: format!(
            "prompt-pack-run-{}-stage-{}",
            request.run_id, request.stage_run_id
        ),
        profile_id,
        model_override,
        max_output_tokens,
        messages: vec![
            LlmMessage {
                role: "system".to_string(),
                content: "Return strict JSON for the YouTube Summary transcript analysis stage. Use only refs from the provided registries.".to_string(),
            },
            LlmMessage {
                role: "user".to_string(),
                content: format!(
                    "Analyze the frozen transcript and return exactly one strict JSON object matching this shape:\n\
                     {{\n\
                     \"stage_io_version\": \"1.0\",\n\
                     \"schema_version\": \"1.0\",\n\
                     \"stage\": \"youtube_summary/transcript_analysis\",\n\
                     \"video_candidate\": {{\n\
                     \"summary_text\": \"readable narrative summary\",\n\
                     \"segment_candidates\": [],\n\
                     \"key_point_candidates\": [{{ \"text\": \"point\", \"segment_candidate_index\": 0, \"material_refs\": [\"allowed material ref\"] }}],\n\
                     \"quote_candidates\": [{{ \"text\": \"short quote\", \"segment_candidate_index\": 0, \"material_refs\": [\"allowed material ref\"] }}],\n\
                     \"action_item_candidates\": [],\n\
                     \"open_question_candidates\": []\n\
                     }},\n\
                     \"claim_candidates\": [{{ \"text\": \"claim\", \"material_refs\": [\"allowed material ref\"] }}],\n\
                     \"evidence_fragment_candidates\": [{{ \"text\": \"evidence quote or paraphrase\", \"quote_candidate_index\": 0, \"material_refs\": [\"allowed material ref\"] }}],\n\
                     \"warning_candidates\": []\n\
                     }}\n\n\
                     summary_text must be a readable narrative summary of the video, not a terse label. {}\n\n\
                     Do not include backend-owned refs or IDs such as segment_ref, key_point_ref, quote_ref, claim_id, evidence_id, source_ref_id, segment_id, key_point_id, quote_id, action_item_id, or open_question_id. For optional candidate-to-candidate linkage, use only zero-based segment_candidate_index and quote_candidate_index. Omit candidate index fields when no clear candidate link exists. Use material_refs only from allowed_material_refs in the frozen input. Do not rename fields. Do not wrap the JSON in Markdown.\n\n\
                     Frozen stage input JSON:\n{}",
                    summary_prompt,
                    request.prompt_input_json
                ),
            },
        ],
    }
}

pub(super) fn build_synthesis_llm_request(
    run_id: i64,
    stage_run_id: i64,
    prompt_input_json: String,
    profile_id: Option<String>,
    model_override: Option<String>,
    max_output_tokens: Option<i64>,
) -> LlmChatRequest {
    LlmChatRequest {
        request_id: format!("prompt-pack-run-{run_id}-stage-{stage_run_id}"),
        profile_id,
        model_override,
        max_output_tokens,
        messages: vec![
            LlmMessage {
                role: "system".to_string(),
                content: "Return strict JSON for the YouTube Summary synthesis stage. Produce a synthesis_candidate only; the backend assigns canonical IDs and traversal fields.".to_string(),
            },
            LlmMessage {
                role: "user".to_string(),
                content: format!(
                    "Synthesize the transcript-analysis candidates into one strict JSON object with stage_io_version, schema_version, stage, synthesis_candidate, limitations, and warning_candidates.\n\nRequired synthesis_candidate shape:\n{{\n  \"summary_text\": \"readable synthesis summary\",\n  \"cross_video_themes\": [{{ \"theme_text\": \"theme\", \"source_refs\": [\"source_ref_1\"], \"claim_refs\": [], \"evidence_refs\": [] }}],\n  \"common_claims\": [],\n  \"contradictions_across_videos\": []\n}}\n\nsummary_text must be a readable synthesis summary, not a terse label. Write 3 to 5 paragraphs in the requested output_language, explaining the shared themes, meaningful differences, and combined takeaway across the analyzed videos. Keep it grounded in the transcript-analysis candidates and canonical_graph; do not copy long transcript passages.\n\nThe input wrapper field source_ref_id may be used only for reasoning. Do not copy the key source_ref_id into the output. Use only source_refs from allowed_refs.source_refs, claim_refs from allowed_refs.claim_refs, and evidence_refs from allowed_refs.evidence_refs. You may use segment_refs, key_point_refs, and quote_refs from allowed_refs only for reasoning over canonical_graph. Do not emit segment_refs, key_point_refs, or quote_refs in the output. Leave claim_refs or evidence_refs empty when no supporting allowed ref exists. Do not include backend-owned IDs or keys such as source_ref_id, theme_id, common_claim_id, contradiction_id, claim_id, evidence_id, video_id, section_id, or synthesis_item_id. Do not wrap the JSON in Markdown.\n\nSynthesis input JSON:\n{}",
                    prompt_input_json
                ),
            },
        ],
    }
}

pub(super) fn gem_part_request_suffix(part: GemAnalysisPart) -> String {
    format!("gem-{}", part.slug())
}

pub(super) fn gem_part_repair_request_suffix(part: GemAnalysisPart, attempt_number: i64) -> String {
    format!("gem-{}-repair-{attempt_number}", part.slug())
}

fn gem_part_output_budget(part: GemAnalysisPart) -> i64 {
    match part {
        GemAnalysisPart::Comments => 4_096,
        GemAnalysisPart::Passport | GemAnalysisPart::DeepRecap => 8_192,
    }
}

pub(super) fn gem_analysis_part_max_output_tokens(
    part: GemAnalysisPart,
    model_output_limit: Option<i64>,
) -> Option<i64> {
    transcript_analysis_max_output_tokens(gem_part_output_budget(part), model_output_limit)
}

pub(super) fn build_gem_analysis_part_llm_request(
    request: &GemAnalysisPartStageExecutionRequest,
    profile_id: Option<String>,
    model_override: Option<String>,
    max_output_tokens: Option<i64>,
) -> LlmChatRequest {
    LlmChatRequest {
        request_id: format!(
            "prompt-pack-run-{}-stage-{}-{}",
            request.run_id,
            request.stage_run_id,
            gem_part_request_suffix(request.part)
        ),
        profile_id,
        model_override,
        max_output_tokens,
        messages: vec![
            LlmMessage {
                role: "system".to_string(),
                content: "Return strict JSON for one Gem analysis part. Do not include Markdown fences, prose outside JSON, comments, or backend-owned IDs. Put the complete Russian Markdown report in the markdown field.".to_string(),
            },
            LlmMessage {
                role: "user".to_string(),
                content: format!(
                    "Return exactly one strict JSON object:\n\
                     {{\n\
                     \"part\": \"{}\",\n\
                     \"markdown\": \"<full Russian Markdown report>\"\n\
                     }}\n\
                     Use only the input material provided below for this part. Do not use outputs from other Gem analysis parts. Do not invent timestamps, metadata, source titles, subscriber counts, metrics, or links. If a requested item is unavailable in the provided material, write `Недоступно во входных данных`. If transcript input has no `[MM:SS]` timestamps, state that timestamps are unavailable in the input and do not create approximate timestamps. For fact-checking, do not fabricate sources or URLs; if external verification is unavailable in the current runtime, explicitly state that limitation. Do not start markdown with # or ##; the backend assembler owns the top-level report title and part headings. Start internal headings at ###, use #### for nested headings, and avoid leading/trailing horizontal rules.\n\n\
                     Gem analysis part input JSON:\n{}",
                    request.part.as_str(),
                    request.prompt_input_json
                ),
            },
        ],
    }
}

pub(super) fn build_gem_analysis_part_repair_llm_request(
    request: &GemAnalysisPartRepairRequest,
    profile_id: Option<String>,
    model_override: Option<String>,
    max_output_tokens: Option<i64>,
) -> LlmChatRequest {
    LlmChatRequest {
        request_id: format!(
            "prompt-pack-run-{}-stage-{}-{}",
            request.run_id,
            request.stage_run_id,
            gem_part_repair_request_suffix(request.part, request.attempt_number)
        ),
        profile_id,
        model_override,
        max_output_tokens,
        messages: vec![
            LlmMessage {
                role: "system".to_string(),
                content: "Return strict JSON for one Gem analysis part. Do not include Markdown fences, prose outside JSON, comments, or backend-owned IDs. Put the complete Russian Markdown report in the markdown field.".to_string(),
            },
            LlmMessage {
                role: "user".to_string(),
                content: format!(
                    "Repair the invalid Gem analysis part output for part `{}`.\n\n\
                     Parser/validator error:\n{}\n\n\
                     Original Gem part input JSON:\n{}\n\n\
                     Invalid provider output:\n{}\n\n\
                     Return exactly one strict JSON object:\n\
                     {{\n\
                     \"part\": \"{}\",\n\
                     \"markdown\": \"<full Russian Markdown report>\"\n\
                     }}\n\
                     Preserve useful Markdown content from the invalid output when possible. Use only the original Gem part input and the invalid output. Do not add prose outside JSON.",
                    request.part.as_str(),
                    request.error_message,
                    request.prompt_input_json,
                    request.raw_output,
                    request.part.as_str(),
                ),
            },
        ],
    }
}

pub(super) fn build_json_repair_llm_request(
    request: &JsonRepairStageExecutionRequest,
    profile_id: Option<String>,
    model_override: Option<String>,
    max_output_tokens: Option<i64>,
) -> LlmChatRequest {
    LlmChatRequest {
        request_id: format!(
            "prompt-pack-run-{}-stage-{}-repair-{}",
            request.run_id, request.stage_run_id, request.attempt_number
        ),
        profile_id,
        model_override,
        max_output_tokens,
        messages: vec![
            LlmMessage {
                role: "system".to_string(),
                content: "Repair invalid provider JSON for a YouTube Summary pipeline stage. Return exactly one strict JSON object. Do not add Markdown, prose, comments, or backend-owned IDs.".to_string(),
            },
            LlmMessage {
                role: "user".to_string(),
                content: format!(
                    "Repair the provider output for stage `{}`.\n\n\
                     Parser/validator error:\n{}\n\n\
                     Original frozen stage input JSON:\n{}\n\n\
                     Invalid provider output:\n{}\n\n\
                     Return only the corrected JSON object for the same stage, schema_version, and stage_io_version. Preserve useful candidate text from the invalid output when possible. If the original output is truncated, complete only the missing JSON structure using the frozen input as context. Do not include backend-owned IDs.",
                    request.stage_name,
                    request.error_message,
                    request.prompt_input_json,
                    request.raw_output
                ),
            },
        ],
    }
}

pub(super) fn transcript_analysis_stage_max_output_token_budget() -> AppResult<i64> {
    stage_max_output_token_budget(TRANSCRIPT_RUNTIME_JSON, "transcript-analysis")
}

pub(super) fn transcript_analysis_stage_max_prompt_token_budget() -> AppResult<i64> {
    stage_max_prompt_token_budget(TRANSCRIPT_RUNTIME_JSON, "transcript-analysis")
}

pub(super) fn transcript_analysis_stage_max_output_token_budget_for_control_preset(
    control_preset: &str,
) -> AppResult<i64> {
    let standard_budget = transcript_analysis_stage_max_output_token_budget()?;
    if control_preset == DETAILED_REPORT_CONTROL_PRESET {
        Ok(standard_budget.max(8_192))
    } else {
        Ok(standard_budget)
    }
}

pub(super) fn synthesis_stage_max_output_token_budget() -> AppResult<i64> {
    stage_max_output_token_budget(SYNTHESIS_RUNTIME_JSON, "synthesis")
}

fn stage_max_prompt_token_budget(asset_json: &str, label: &str) -> AppResult<i64> {
    let asset = serde_json::from_str::<StageRuntimeConfigAsset>(asset_json).map_err(|error| {
        AppError::internal(format!(
            "Parse bundled {label} runtime configuration: {error}"
        ))
    })?;
    asset
        .runtime_configuration
        .and_then(|runtime| runtime.budget_limits)
        .and_then(|budget| budget.max_prompt_tokens)
        .filter(|max_prompt_tokens| *max_prompt_tokens > 0)
        .ok_or_else(|| {
            AppError::internal(format!(
                "Bundled {label} runtime configuration is missing positive max_prompt_tokens"
            ))
        })
}

fn stage_max_output_token_budget(asset_json: &str, label: &str) -> AppResult<i64> {
    let asset = serde_json::from_str::<StageRuntimeConfigAsset>(asset_json).map_err(|error| {
        AppError::internal(format!(
            "Parse bundled {label} runtime configuration: {error}"
        ))
    })?;
    asset
        .runtime_configuration
        .and_then(|runtime| runtime.budget_limits)
        .and_then(|budget| budget.max_output_tokens)
        .filter(|max_output_tokens| *max_output_tokens > 0)
        .ok_or_else(|| {
            AppError::internal(format!(
                "Bundled {label} runtime configuration is missing positive max_output_tokens"
            ))
        })
}

pub(super) fn transcript_analysis_max_output_tokens(
    stage_output_budget: i64,
    model_output_limit: Option<i64>,
) -> Option<i64> {
    Some(match model_output_limit.filter(|limit| *limit > 0) {
        Some(limit) => stage_output_budget.min(limit),
        None => stage_output_budget,
    })
}

pub(super) fn gem_input_cap(model_input_limit: Option<usize>, prompt_budget: i64) -> i64 {
    match model_input_limit
        .and_then(|limit| i64::try_from(limit).ok())
        .filter(|limit| *limit > 0)
    {
        Some(model_limit) => model_limit.min(prompt_budget),
        None => prompt_budget,
    }
}
