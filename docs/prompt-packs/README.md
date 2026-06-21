# Prompt Packs Documentation

Этот каталог содержит рабочую документацию по библиотеке prompt packs,
общему JSON-контракту результатов и pack-specific schemas.

## Основные документы

| Документ | Назначение | Статус |
|---|---|---|
| `PROMPT_PACK_LIBRARY_DECISIONS.md` | Decision log по содержательной библиотеке prompt packs | Рабочая версия |
| `prompt_pack_json_contract_v1_draft.md` | Основной Prompt Pack JSON Contract v1 | Рабочая версия v1 |
| `PROMPT_PACK_JSON_CONTRACT_DECISIONS.md` | Decision log по JSON-контракту | Обновлён по v1 draft |
| `source_type_schemas.md` | Companion document: `source_ref.type_data` schemas | v1.0 |
| `SOURCE_TYPE_SCHEMA_DECISIONS.md` | Decision log по `source_type_schemas.md` | Рабочая версия |
| `fragment_locator_schemas.md` | Companion document: `evidence.locator_data` schemas | v1.0 |
| `FRAGMENT_LOCATOR_SCHEMA_DECISIONS.md` | Decision log по `fragment_locator_schemas.md` | Рабочая версия |
| `schemas/README.md` | Machine-readable JSON Schema placement and v1 semantic schema bundle | v1 semantic |
| `validation_rules.md` | Карта hard errors, warnings, QA и pipeline-level validation rules | v1 draft |
| `validator_manifest.md` | Execution manifest for the first reference validator: phases, inputs, rule groups, fixtures | v1 draft |
| `validator_fixtures.md` | Fixture catalog for reference validator CI and local validation tests | v1 draft |
| `execution_model_graph_assembly_policy.md` | Execution model, graph assembly, healing, quarantine and fragment deduplication policy | v1 draft |
| `stage_io_contracts.md` | Stage-level input/output contracts for prompt-pack execution pipeline | v1 draft |
| `stage_prompt_templates.md` | Provider-neutral prompt skeletons, output JSON shapes, retry prompts, and few-shot rules for LLM stages | v1 draft |
| `model_recommendations.md` | Provider-neutral model class recommendations and escalation policy per LLM stage | v1 draft |
| `runtime_configuration_policy.md` | Boundary policy for runtime config, feature flags, budgets, retry/quarantine and canonical JSON exposure | v1 draft |
| `prompts/v1/README.md` | Provider-specific prompt file layout; OpenAI-compatible stage prompts | v1 draft |
| `prompts/v1/telegram_summary_pack_data_generation.md` | Telegram-specific guidance for rendering `pack_data_generation` prompts | v1 draft |
| `parser-fixtures/v1/README.md` | Raw provider-response parser fixtures for stage outputs | v1 draft |
| `technology_watch_pack_spec.md` | Pack-specific schema для `technology_watch` | v1 baseline |
| `TECHNOLOGY_WATCH_PACK_DECISIONS.md` | Decision log по `technology_watch` | Рабочая версия |
| `youtube_summary_pack_spec.md` | Pack-specific schema для `youtube_summary` | v1 draft |
| `YOUTUBE_SUMMARY_PACK_DECISIONS.md` | Decision log по `youtube_summary` | Рабочая версия |
| `telegram_summary_pack_spec.md` | Pack-specific schema для `telegram_summary` | v1 draft |
| `TELEGRAM_SUMMARY_PACK_DECISIONS.md` | Decision log по `telegram_summary` | Рабочая версия |

## Reference Validator Skeleton

Current executable skeleton:

```text
scripts/prompt_pack_validator/
scripts/prompt_pack_validator_tests.py
```

Current fixture baseline:

```text
docs/prompt-packs/fixtures/v1/fixture_manifest.json
```

Run from `research-control-deck\scripts` with an absolute manifest path in
sandboxed environments:

```text
python -m prompt_pack_validator validate-fixtures --manifest F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\fixtures\v1\fixture_manifest.json
```

The skeleton validates the mandatory fixture baseline, selected parser fixtures,
machine-readable JSON Schema local shape for normalized artifact boundaries, and
a growing subset of graph/reference/pipeline rules. It is a reference
implementation aid; the prose contract and validation rule documents remain
authoritative.

## Prompt Execution Artifacts

Provider-neutral stage prompts are defined in `stage_prompt_templates.md`.
Provider-specific renderings are checked in for:

```text
prompts/v1/openai-compatible/fragment_candidate_mining.prompt.json
prompts/v1/openai-compatible/claim_extraction.prompt.json
prompts/v1/openai-compatible/claim_linking.prompt.json
prompts/v1/openai-compatible/pack_data_generation.prompt.json
prompts/v1/openai-compatible/final_synthesis.prompt.json
prompts/v1/openai-compatible/retry_repair.prompt.json
```

Raw provider-response parser fixtures live in:

```text
parser-fixtures/v1/fragment_candidate_mining/
parser-fixtures/v1/claim_extraction/
parser-fixtures/v1/claim_linking/
parser-fixtures/v1/pack_data_generation/
parser-fixtures/v1/final_synthesis/
parser-fixtures/v1/retry_repair/
```

The parser fixtures are executable through the parser-fixture runner. They are
intentionally separate from the mandatory `fixtures/v1/fixture_manifest.json`
baseline, which starts at already parsed `stage_output` artifacts.

Parser fixture runner command from `research-control-deck\scripts`:

```text
python -m prompt_pack_validator validate-parser-fixtures --fixtures-dir F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\parser-fixtures\v1\fragment_candidate_mining
python -m prompt_pack_validator validate-parser-fixtures --fixtures-dir F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\parser-fixtures\v1\claim_extraction
python -m prompt_pack_validator validate-parser-fixtures --fixtures-dir F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\parser-fixtures\v1\claim_linking
python -m prompt_pack_validator validate-parser-fixtures --fixtures-dir F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\parser-fixtures\v1\pack_data_generation
python -m prompt_pack_validator validate-parser-fixtures --fixtures-dir F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\parser-fixtures\v1\final_synthesis
python -m prompt_pack_validator validate-parser-fixtures --fixtures-dir F:\work\Develop\TestGUI\research-control-deck\docs\prompt-packs\parser-fixtures\v1\retry_repair
```

Current open directions after this layer:

- Add parser fixtures for the next stage prompts once those prompt renders are
  checked in.

## Baseline v1

Текущий baseline v1 включает:

- общий контракт `prompt_pack_json_contract_v1_draft.md`;
- companion schema `source_type_schemas.md`;
- companion schema `fragment_locator_schemas.md`;
- machine-readable schema placement `schemas/README.md`;
- validation map `validation_rules.md`;
- validator manifest `validator_manifest.md`;
- validator fixtures `validator_fixtures.md`;
- execution policy `execution_model_graph_assembly_policy.md`;
- stage I/O contracts `stage_io_contracts.md`;
- stage prompt templates `stage_prompt_templates.md`;
- model class recommendations `model_recommendations.md`;
- runtime configuration boundary policy `runtime_configuration_policy.md`;
- provider prompt renders under `prompts/v1/openai-compatible/`;
- parser fixture sets under `parser-fixtures/v1/`;
- pack-specific schema `technology_watch_pack_spec.md`.

Этот набор достаточен для реализации первого pack `technology_watch`.
`youtube_summary_pack_spec.md` добавлен как второй pack-specific draft поверх того же
baseline.
`telegram_summary_pack_spec.md` добавлен как третий pack-specific draft для
Telegram channel/chat summary поверх того же baseline.
Открытые вопросы, оставшиеся в документах, считаются non-blocking для v1.

## Текущая архитектура

Общий контракт разделяет результат на два слоя:

- `outputs` — readable и pack-specific форма результата;
- `claims`, `evidence`, `source_refs`, `claim_relations` — проверяемость и трассировка.

Companion documents уточняют типизированные вложенные объекты:

- `source_type_schemas.md` валидирует `source_ref.type_data`;
- `fragment_locator_schemas.md` валидирует `evidence.locator_data`;
- `schemas/README.md` and `schemas/v1/schema_manifest.json` fix the
  machine-readable JSON Schema semantic bundle under `schemas/v1/`;
- `validation_rules.md` классифицирует hard errors, warnings, QA и pipeline-level проверки;
- `validator_manifest.md` фиксирует execution phases, inputs, rule groups и CI
  fixture expectations для первого reference validator;
- `validator_fixtures.md` фиксирует fixture directory layout, naming,
  mandatory fixture matrix и expected findings;
- `execution_model_graph_assembly_policy.md` описывает ownership, topological generation,
  graph healing, quarantine, pre-contract fragment registry и deduplication;
- `stage_io_contracts.md` фиксирует internal input/output payloads для stage-level
  генерации, repair и assembly;
- `stage_prompt_templates.md` фиксирует provider-neutral prompt skeletons,
  expected narrow JSON outputs и few-shot rules для LLM stages;
- `model_recommendations.md` фиксирует provider-neutral `model_class`
  рекомендации, escalation policy и runtime routing shape для LLM stages;
- `runtime_configuration_policy.md` фиксирует границу между runtime config,
  audit/telemetry и canonical result JSON;
- pack-specific specs валидируют `outputs.pack_data[pack_id]`.

## Первый pack

`technology_watch` использует technology-centric модель:

```json
{
  "pack_data": {
    "technology_watch": {
      "technologies": []
    }
  }
}
```

Подробная схема находится в `technology_watch_pack_spec.md`.
Принятые решения по pack-у кратко зафиксированы в
`TECHNOLOGY_WATCH_PACK_DECISIONS.md`.

Открытые вопросы v1:

- OQ-TW-02 — нужны ли `signal_refs` в barriers/risks;
- OQ-TW-03 — нужны ли `technology_refs` в `sections.items`.

## Второй pack

`youtube_summary` использует video-centric модель:

```json
{
  "pack_data": {
    "youtube_summary": {
      "videos": [],
      "synthesis": null
    }
  }
}
```

Подробная схема находится в `youtube_summary_pack_spec.md`.
Принятые решения по pack-у кратко зафиксированы в
`YOUTUBE_SUMMARY_PACK_DECISIONS.md`.

Открытые вопросы v1:

- ~~OQ-YS-01~~ — закрыт через `speaker_id` в `notable_quote`;
- OQ-YS-02 — достаточно ли текущей granularity `synthesis`;
- OQ-YS-03 — нужен ли отдельный playlist-level context.

## Третий pack

`telegram_summary` использует Telegram message-centric модель с явным
pack-local индексом сообщений:

```json
{
  "pack_data": {
    "telegram_summary": {
      "source_shape": "mixed",
      "sources": [],
      "message_refs": [],
      "digest": null,
      "timeline": [],
      "topics": [],
      "key_messages": [],
      "threads": [],
      "claims": [],
      "forwarded_items": [],
      "message_quality_signals": [],
      "cross_source_synthesis": null,
      "limitations": []
    }
  }
}
```

Подробная схема находится в `telegram_summary_pack_spec.md`.
Принятые решения по pack-у кратко зафиксированы в
`TELEGRAM_SUMMARY_PACK_DECISIONS.md`.

Открытые вопросы v1:

- OQ-TS-01 — нужна ли отдельная schema-форма для recursive summaries очень
  длинных reply chains;
- OQ-TS-02 — должны ли веса importance scoring быть runtime-configurable,
  prompt-guided или фиксироваться в pack runtime profile.

## Открытые направления

- JSON Schema placement: где хранить machine-readable schemas для core, companion и pack-specific документов.
- Few-shot examples и stage templates для каждого стартового pack.
