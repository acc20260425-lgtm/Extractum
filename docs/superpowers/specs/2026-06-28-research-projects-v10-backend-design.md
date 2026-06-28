# Research Projects v10 — подготовка Rust-бэкенда под дизайн

**Дата:** 2026-06-28
**Статус:** согласовано (brainstorming), готово к написанию плана реализации
**Целевой макет:** [`reference/Tauri MCP Bridge connection 4/Research Projects v10.dc.html`](../../../reference/Tauri%20MCP%20Bridge%20connection%204/Research%20Projects%20v10.dc.html) (в репозитории; статус источника приведён к контракту бэкенда — `active/syncing/error/unavailable`, бывший `idle` убран)
**Скоуп:** широкий рефакторинг бэкенда «research projects» — данные, новые команды, миграция схемы (закрепление/архив).

## Цель

Сократить разрыв между макетом v10 и текущим бэкендом так, чтобы внедрение дизайна на фронте было «подменой данных», а не борьбой с агрегацией на клиенте. Бэкенд должен отдавать готовые формы для рейла проектов, таблицы источников и тулбара периода.

Явно **вне скоупа:** трассировка прогресса прогона в процентах (см. решение 1).

## Разрыв «дизайн v10 → текущий бэкенд»

| Элемент макета | Нужные данные | Сейчас |
|---|---|---|
| `ProjectRow` (рейл) | имя, кол-во источников, статус, относительное «N назад», точка-цвет, закреплён/архив | `list_projects` отдаёт только `id/name/description/created_at/updated_at`; агрегаты собираются на клиенте; закрепления/архива нет |
| `SourceRow` (таблица) | title, провайдер+точка, материалы, «последний сбор» (`last_synced_at`), статус (active/syncing/error/unavailable), хэндл | `list_project_sources` не отдаёт `last_synced_at`, sync-статус; `subtitle` = `Account #N` вместо хэндла |
| `ProjectToolbar` период | `dataRange {from,to}` = MIN/MAX даты материалов | нет; фронт хардкодит `"All time"` |
| Промпт/Модель селекторы | списки | ✅ уже есть (`list_analysis_prompt_templates`, `list_llm_provider_models`) |
| Фильтры/сортировка таблицы | тип, статус, материалы-диапазон, дата-диапазон, поиск | остаются клиентскими (список источников проекта мал) |

**Ключевой факт по дате материала:** `dataRange` должен совпадать с **реально анализируемым корпусом**,
а не с любым контентом. Основной корпус грузится из `analysis_documents` с фильтром по `published_at` и
`document_kind` ([`load_analysis_document_messages`, corpus.rs:592](../../../src-tauri/src/analysis/corpus.rs)),
а ветка migrated-history — из `items` с доп. фильтрами `content_zstd`/`content_kind`
([corpus.rs:506](../../../src-tauri/src/analysis/corpus.rs)). Поэтому `dataRange` строится **через тот же
источник и фильтры, что corpus loader** (`analysis_documents`, при необходимости `UNION` с migrated-history из
`items`), а **не** из `MIN/MAX(items.published_at)` напрямую — иначе тулбар покажет период, не совпадающий с
анализируемыми данными. Дата сбора (`sources.last_synced_at`) — отдельная величина, нужна только для колонки
«Последний сбор» в `SourceRow`. См. раздел D.

## Согласованные решения

1. **Прогресс %% — удалён из скоупа.** В `ProjectRow` остаётся только бинарный статус «идёт анализ».
   Никаких колонок прогресса в `analysis_runs`, никакого персиста `RunEvent("progress")`.
   `running` определяется наличием активного прогона (`queued`/`running`) — уже доступно.
2. **`dataRange` считается лениво.** Отдельная команда `get_project_data_range` (сигнатура с
   `youtube_corpus_mode`/`include_migrated_history` — см. раздел D), вызывается тулбаром при выборе проекта.
   В список рейла (`ProjectSummary`) диапазон **не** кладётся (тяжёлый запрос на каждую строку не нужен).
3. **Статус источника переиспользует логику И контракт library-catalog** —
   [`catalog_status_for_source()`](../../../src-tauri/src/library_sources/mod.rs) (Active/Syncing/Error/Unavailable
   из `source` + последнего sync-job'а). DTO отдаёт **существующие** значения `LibraryCatalogStatus`
   (`active`/`syncing`/`error`/`unavailable`), помеченные «stable for API» в
   [`docs/value-registry.md`](../../value-registry.md). Новых строк в бэкенде **не вводим**. Макет приведён
   к этим же значениям (Variant A: `idle` заменён на `unavailable`), поэтому единственное отличие — короткая
   подпись `syncing→"sync"` на уровне отображения. `value-registry.md` по статусу источника не меняется.
4. **Новые команды vs правка:** новая read-модель `list_research_projects` (агрегаты);
   у `list_project_sources` поля **дописываются аддитивно** (back-compat). Старый `list_projects`
   остаётся, потребители мигрируют постепенно.
5. **`needs_attention` = только упавший последний прогон** (`failed`). Ошибки синка источников
   видны своими точками статуса в таблице и проект не красят.

## Архитектура изменений (подход 1 — выделенная read-модель)

Прецедент в кодовой базе: `src/archive_read_model.rs`. Чтение (read-модель) и мутации разделены.

### A. Миграция `0012_projects_redesign.sql`

```sql
ALTER TABLE projects ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0;
ALTER TABLE projects ADD COLUMN archived_at INTEGER;            -- NULL = не в архиве
CREATE INDEX IF NOT EXISTS idx_projects_pinned_archived
    ON projects(pinned DESC, archived_at, updated_at DESC);
```

- `pinned` — закрепление в рейле (секция «Закреплённые»).
- `archived_at` — момент архивации; `NULL` = активный проект. Архив — отдельная сворачиваемая секция рейла.

**Регистрация миграции (для плана).** Положить `.sql` в `migrations/` недостаточно — миграции собираются
кодом в [`migrations.rs`](../../../src-tauri/src/migrations.rs). План обязан включить все шаги по образцу
`projects_mvp_migration()`: (1) `const` с version/description/`include_str!` SQL; (2) функция
`projects_redesign_migration() -> Migration`; (3) добавить её вызов в `build_migrations()`
([migrations.rs:259](../../../src-tauri/src/migrations.rs)) **после** `prompt_pack_stage_browser_provenance_migration()`;
(4) тест на регистрацию и применение на baseline (дефолты `pinned=0`, `archived_at=NULL`).

### B. Новый модуль `src/projects/read_model.rs`

Команда `list_research_projects` → `Vec<ProjectSummary>`. Агрегаты считаются в SQL без N+1.

> **Требование к запросу (анти-fanout):** каждый агрегат (`source_count`, `material_count`, `last_run_at`,
> наличие активного/упавшего прогона) считается **отдельным коррелированным подзапросом или CTE**, а не
> одним плоским `JOIN` по `project_sources`/`items`/`analysis_runs` — иначе строки перемножатся и счётчики
> завысятся. Где плоский join неизбежен — использовать `COUNT(DISTINCT ...)`.

```rust
pub struct ProjectSummary {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub source_count: i64,        // COUNT(DISTINCT project_sources.source_id)
    pub material_count: i64,      // собранные материалы: SUM(COUNT(items.content_zstd)) по источникам проекта
    pub status: ProjectStatus,    // running | empty | needs_attention | ready (см. приоритет ниже)
    pub last_run_at: Option<i64>, // created_at последнего прогона проекта
    pub pinned: bool,
    pub archived: bool,           // archived_at IS NOT NULL
    pub updated_at: i64,
}
```

**`material_count` — это собранные материалы** (тот же счётчик, что колонка «Материалы» в `SourceRow`:
`COUNT(items.content_zstd)` на источник, суммируется по проекту). Это **не** число анализируемых
`analysis_documents` и не связано с `dataRange` — `dataRange` про ось дат анализируемого корпуса, а
`material_count` про объём собранного контента. Разные величины, не путать.

Правила `status` (в порядке приоритета, **`running` первым**):
1. есть прогон со статусом `queued`/`running` → `running`;
2. `source_count == 0` → `empty`;
3. **последний** прогон проекта имеет статус `failed` → `needs_attention`;
4. иначе → `ready`.

`running` стоит **выше** `empty` намеренно: если источники удалить во время активного прогона, проект не должен
терять индикатор работы (прогон крутится на снапшоте данных). «Последний» прогон детерминирован:
`ORDER BY created_at DESC, id DESC LIMIT 1` (tie-breaker по `id`, т.к. `analysis_runs.created_at` — целые
секунды и два прогона могут совпасть).

**Контракт (для плана).** `ProjectStatus` (`ready/running/needs_attention/empty`) сейчас в
[`docs/value-registry.md`](../../value-registry.md) помечен как frontend-derived/unstable. Перенос derive в
бэкенд делает его частью API — план **обязан** обновить реестр: пометить эти значения как backend-derived и
зафиксировать wire-формат (snake_case).

`data_range` в `ProjectSummary` **не входит** (решение 2).

### C. Расширение `ProjectSourceRecord` (аддитивно, в `src/projects.rs`)

Дописать в `list_project_sources` без удаления/переименования существующих полей:

```rust
pub last_synced_at: Option<i64>,        // sources.last_synced_at → колонка «Последний сбор»
pub sync_status: LibraryCatalogStatus,  // active | syncing | error | unavailable (тот же контракт, что в каталоге)
pub handle: Option<String>,             // хэндл (@channel / youtube · канал) вместо "Account #N"
```

`sync_status` отдаёт **существующие** значения `LibraryCatalogStatus` без переименования (макет уже на тех же
значениях, Variant A). Единственное отличие отображения — короткая подпись `syncing→"sync"` на стороне UI.

**Видимость типа (решение для плана).** Сейчас `LibraryCatalogStatus` — `pub(crate)` enum
([models.rs:56](../../../src-tauri/src/library_sources/models.rs)), а `ProjectSourceRecord` — публичный DTO
([projects.rs:16](../../../src-tauri/src/projects.rs)). Класть `pub(crate)`-тип в публичное поле нельзя
(private-in-public). План фиксирует: **поднять `LibraryCatalogStatus` до `pub`** и оформить как
именованный API-тип статуса источника (он и так сериализуется во фронт командой каталога). Альтернатива —
завести отдельный публичный enum/newtype с теми же wire-значениями — отвергнута как дублирование контракта.

**Граница модулей (для плана).** `catalog_status_for_source()` сейчас — приватная `fn` внутри
`library_sources` ([mod.rs:186](../../../src-tauri/src/library_sources/mod.rs)). План должен явно включить:
(1) поднять её видимость до `pub(crate)` (или вынести в общий модуль), (2) получить **последний sync-job
источника** для `list_project_sources` тем же способом, что каталог
([`latest_catalog_jobs_by_source`, mod.rs:129](../../../src-tauri/src/library_sources/mod.rs)) — иначе
реализация упрётся в модульные границы и логику придётся дублировать.

Источник хэндла: `sources.external_id`/метаданные (точная форма уточняется при реализации —
переиспользовать существующий хелпер отображения источника, если есть).

### D. Ленивый диапазон данных — `get_project_data_range`

```rust
pub struct ProjectDataRange { pub from: Option<i64>, pub to: Option<i64> } // оба NULL если материалов нет

#[tauri::command]
pub async fn get_project_data_range(
    handle,
    project_id: i64,
    youtube_corpus_mode: Option<String>,   // те же параметры, что у start_project_analysis
    include_migrated_history: bool,
) -> AppResult<ProjectDataRange>;
```

**Почему параметры обязательны.** corpus loader меняет набор `document_kind` по `youtube_corpus_mode`
(`youtube_transcript` ± `youtube_description`/`youtube_comment`, [corpus.rs:626](../../../src-tauri/src/analysis/corpus.rs))
и добавляет migrated-history ветку по `include_migrated_history`. `start_project_analysis` уже принимает оба
([projects.rs:422](../../../src-tauri/src/projects.rs)). Если диапазон считать без них, он разойдётся с реально
анализируемым корпусом при тех же настройках. **Реализация переиспользует
[`resolve_analysis_sources`](../../../src-tauri/src/analysis/corpus.rs) (corpus.rs:203)** для получения
`source_ids` + `source_type` проекта (и единой проверки mixed-provider), затем строит MIN/MAX по тем же
фильтрам.

> **Range фильтрует по resolved `source_ids`, НЕ по `project_sources`.** `resolve_analysis_sources`
> раскрывает youtube-playlist в `source_id` связанных видео ([`push_scope_source`, corpus.rs:313](../../../src-tauri/src/analysis/corpus.rs)),
> а `project_sources` хранит сам playlist-источник. Если джойнить по `project_sources.source_id`, видео
> плейлиста выпадут из диапазона. Поэтому фильтр — `WHERE d.source_id IN (<resolved source_ids>)`.

> **Валидация `include_migrated_history`.** `include_migrated_history=true` допустим только для Telegram —
> `start_project_analysis` это уже проверяет через
> [`resolve_analysis_telegram_history_scope`, report.rs:64](../../../src-tauri/src/analysis/report.rs).
> Команда range **обязана** повторить это поведение: переиспользовать тот же helper (поднять его видимость до
> `pub(crate)`) либо вернуть такую же validation error, чтобы тулбар и анализ не расходились.

Диапазон **зеркалит источник и фильтры corpus loader**, а не берёт сырой `items`. Основной корпус —
`analysis_documents` (с тем же `document_kind`-фильтром, что в
[`load_analysis_document_messages`](../../../src-tauri/src/analysis/corpus.rs)):

```sql
-- основной корпус (current); :ids = resolved source_ids из resolve_analysis_sources
SELECT MIN(d.published_at) AS from_ts, MAX(d.published_at) AS to_ts
FROM analysis_documents d
WHERE d.source_id IN (:ids)
  -- document_kind в наборе, который грузит corpus loader для типа источника
  -- (telegram_message / youtube_transcript[/description/comment])
```

Если включена migrated-history (только Telegram, см. валидацию выше), диапазон расширяется `UNION` с
`items.published_at` по той же ветке фильтров, что `fetch_telegram_corpus_rows(..., include_migrated_rows=true)`
([corpus.rs:506](../../../src-tauri/src/analysis/corpus.rs)): `items.source_id IN (:ids)`,
`item_kind='telegram_message'`, `is_migrated_history=1`, `migration_domain='migrated_from_chat'`,
`content_zstd IS NOT NULL`, `content_kind IN ('text_only','text_with_media')`. Итоговый range = MIN/MAX по объединению.

**В плане реализации** вынести список `document_kind`/фильтров в общий хелпер вместе с corpus loader,
чтобы range и реально анализируемый корпус не разъехались при будущих правках.

Тулбар строит пресеты периода из этого диапазона и клампит кастомный выбор по его краям.

### E. Мутации закрепления/архива (в `src/projects.rs`)

```rust
#[tauri::command] pub async fn set_project_pinned(handle, project_id, pinned: bool) -> AppResult<()>;
#[tauri::command] pub async fn set_project_archived(handle, project_id, archived: bool) -> AppResult<()>;
```

`set_project_archived(true)` проставляет `archived_at = now()`, `false` — `NULL`.
Оба обновляют `updated_at`. Регистрируются в `invoke_handler` ([`src/lib.rs:206`](../../../src-tauri/src/lib.rs)).

### F. Что НЕ трогаем

- Фильтры/сортировка/поиск таблицы источников — остаются на клиенте (список источников проекта мал; YAGNI).
  Вынос в бэкенд — только если профилирование покажет проблему.
- Прогресс прогона — вне скоупа (решение 1).
- Старые `list_projects` / существующие потребители — без изменений в этом заходе.

## Изоляция и тестируемость

| Юнит | Назначение | Тесты |
|---|---|---|
| `read_model.rs::list_research_projects` | агрегаты + derive статуса | SQL-агрегаты без fanout, все 4 ветки статуса, **`running` выигрывает у `empty`** (источники удалены при active run), tie-breaker последнего прогона, архивный проект |
| `get_project_data_range` | range по `analysis_documents` (+UNION migrated) | проект без материалов (NULL/NULL), несколько источников, **playlist раскрыт в видео (range покрывает видео, не сам playlist)**, влияние `youtube_corpus_mode`, **migrated-history для не-Telegram → validation error**, совпадение с corpus loader |
| `ProjectSourceRecord` расширение | last_synced_at / sync_status / handle | каждое значение `LibraryCatalogStatus` (active/syncing/error/unavailable), отсутствие last_synced_at |
| `catalog_status_for_source` extraction | pub(crate) + latest source job | каталог и проект дают одинаковый статус для одного источника |
| миграция 0012 + регистрация | колонки pinned/archived_at | зарегистрирована в `build_migrations()`; применяется на baseline; дефолты |
| `set_project_pinned` / `set_project_archived` | мутации флагов | toggle, обновление updated_at, несуществующий проект |
| контракты в `value-registry.md` | `ProjectStatus` → backend-derived | реестр обновлён; wire-значения snake_case зафиксированы |

Существующие тесты `projects.rs` (создание/идемпотентность/удаление) должны продолжать проходить —
расширение `ProjectSourceRecord` аддитивно.

## Открытые мелочи для этапа реализации

- Точная форма `handle` для telegram/youtube — переиспользовать существующий хелпер отображения, если найдётся;
  иначе собрать из `external_id` + `source_subtype`.
- Presentation-подпись фронта `syncing→"sync"` — на стороне UI; бэкенд отдаёт канонические `LibraryCatalogStatus`.

## Следующий шаг

Перейти к скилу writing-plans — детальный план реализации: миграция + регистрация в `migrations.rs` →
поднять `LibraryCatalogStatus`/`catalog_status_for_source` до `pub`/`pub(crate)` → read-модель
`list_research_projects` → расширение `ProjectSourceRecord` → ленивый `get_project_data_range` (reuse
`resolve_analysis_sources`) → мутации pinned/archived → регистрация команд в `lib.rs` → обновить
`value-registry.md` → тесты.
