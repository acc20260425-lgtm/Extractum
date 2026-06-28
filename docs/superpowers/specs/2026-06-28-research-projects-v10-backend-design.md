# Research Projects v10 — подготовка Rust-бэкенда под дизайн

**Дата:** 2026-06-28
**Статус:** согласовано (brainstorming), готово к написанию плана реализации
**Целевой макет:** `Research Projects v10.dc.html` (папка `Tauri MCP Bridge connection 3`)
**Скоуп:** широкий рефакторинг бэкенда «research projects» — данные, новые команды, миграция схемы (закрепление/архив).

## Цель

Сократить разрыв между макетом v10 и текущим бэкендом так, чтобы внедрение дизайна на фронте было «подменой данных», а не борьбой с агрегацией на клиенте. Бэкенд должен отдавать готовые формы для рейла проектов, таблицы источников и тулбара периода.

Явно **вне скоупа:** трассировка прогресса прогона в процентах (см. решение 1).

## Разрыв «дизайн v10 → текущий бэкенд»

| Элемент макета | Нужные данные | Сейчас |
|---|---|---|
| `ProjectRow` (рейл) | имя, кол-во источников, статус, относительное «N назад», точка-цвет, закреплён/архив | `list_projects` отдаёт только `id/name/description/created_at/updated_at`; агрегаты собираются на клиенте; закрепления/архива нет |
| `SourceRow` (таблица) | title, провайдер+точка, материалы, «последний сбор» (`last_synced_at`), статус (active/sync/idle/error), хэндл | `list_project_sources` не отдаёт `last_synced_at`, sync-статус; `subtitle` = `Account #N` вместо хэндла |
| `ProjectToolbar` период | `dataRange {from,to}` = MIN/MAX даты материалов | нет; фронт хардкодит `"All time"` |
| Промпт/Модель селекторы | списки | ✅ уже есть (`list_analysis_prompt_templates`, `list_llm_provider_models`) |
| Фильтры/сортировка таблицы | тип, статус, материалы-диапазон, дата-диапазон, поиск | остаются клиентскими (список источников проекта мал) |

**Ключевой факт по дате материала:** анализ фильтрует корпус по `items.published_at`
([`src-tauri/src/analysis/corpus.rs:459`](../../../src-tauri/src/analysis/corpus.rs)). Поэтому `dataRange`
для пресетов периода строится из `MIN/MAX(items.published_at)`, а не из даты сбора `ingested_at`/`last_synced_at`.
Дата сбора (`sources.last_synced_at`) используется только для колонки «Последний сбор» в `SourceRow`.

## Согласованные решения

1. **Прогресс %% — удалён из скоупа.** В `ProjectRow` остаётся только бинарный статус «идёт анализ».
   Никаких колонок прогресса в `analysis_runs`, никакого персиста `RunEvent("progress")`.
   `running` определяется наличием активного прогона (`queued`/`running`) — уже доступно.
2. **`dataRange` считается лениво.** Отдельная команда `get_project_data_range(project_id)`,
   вызывается тулбаром при выборе проекта. В список рейла (`ProjectSummary`) диапазон **не** кладётся
   (тяжёлый `MIN/MAX(published_at)` по большим источникам не нужен на каждую строку).
3. **Статус источника переиспользует логику library-catalog** —
   [`catalog_status_for_source()`](../../../src-tauri/src/library_sources/mod.rs) (Active/Syncing/Error/Unavailable
   из `source` + последнего sync-job'а). Один и тот же источник показывает одинаковый статус в библиотеке и в проекте.
   Второй маппинг не вводим.
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

### B. Новый модуль `src/projects/read_model.rs`

Команда `list_research_projects` → `Vec<ProjectSummary>`. Агрегаты считаются в SQL за один проход
(подзапросы/`LEFT JOIN` по `project_sources`, `items`, `analysis_runs`), без N+1.

```rust
pub struct ProjectSummary {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub source_count: i64,        // COUNT(project_sources)
    pub material_count: i64,      // SUM(item_count по источникам проекта)
    pub status: ProjectStatus,    // ready | running | needs_attention | empty
    pub last_run_at: Option<i64>, // MAX(analysis_runs.created_at) для проекта
    pub pinned: bool,
    pub archived: bool,           // archived_at IS NOT NULL
    pub updated_at: i64,
}
```

Правила `status` (в порядке приоритета):
1. `source_count == 0` → `empty`;
2. есть прогон со статусом `queued`/`running` → `running`;
3. последний прогон по `created_at` имеет статус `failed` → `needs_attention`;
4. иначе → `ready`.

`data_range` в `ProjectSummary` **не входит** (решение 2).

### C. Расширение `ProjectSourceRecord` (аддитивно, в `src/projects.rs`)

Дописать в `list_project_sources` без удаления/переименования существующих полей:

```rust
pub last_synced_at: Option<i64>,  // sources.last_synced_at → колонка «Последний сбор»
pub sync_status: SourceSyncStatus, // переиспользует catalog_status_for_source()
pub handle: Option<String>,        // хэндл (@channel / youtube · канал) вместо "Account #N"
```

`sync_status` маппится из `LibraryCatalogStatus`: `Active→active`, `Syncing→sync`,
`Error→error`, `Unavailable→idle`. Маппинг фиксируется в одном месте, не дублируется.

Источник хэндла: `sources.external_id`/метаданные (точная форма уточняется при реализации —
переиспользовать существующий хелпер отображения источника, если есть).

### D. Ленивый диапазон данных — `get_project_data_range`

```rust
pub struct ProjectDataRange { pub from: Option<i64>, pub to: Option<i64> } // оба NULL если материалов нет

#[tauri::command]
pub async fn get_project_data_range(handle, project_id) -> AppResult<ProjectDataRange>;
```

```sql
SELECT MIN(items.published_at), MAX(items.published_at)
FROM items
JOIN project_sources ps ON ps.source_id = items.source_id
WHERE ps.project_id = ?;
```

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
| `read_model.rs::list_research_projects` | агрегаты + derive статуса | SQL-агрегаты, все 4 ветки статуса, пустой/архивный проект |
| `get_project_data_range` | MIN/MAX published_at | проект без материалов (NULL/NULL), несколько источников, границы |
| `ProjectSourceRecord` расширение | last_synced_at / sync_status / handle | маппинг каждого `LibraryCatalogStatus`, отсутствие last_synced_at |
| миграция 0012 | колонки pinned/archived_at | применяется на baseline; дефолты |
| `set_project_pinned` / `set_project_archived` | мутации флагов | toggle, обновление updated_at, несуществующий проект |

Существующие тесты `projects.rs` (создание/идемпотентность/удаление) должны продолжать проходить —
расширение `ProjectSourceRecord` аддитивно.

## Открытые мелочи для этапа реализации

- Точная форма `handle` для telegram/youtube — переиспользовать существующий хелпер отображения, если найдётся;
  иначе собрать из `external_id` + `source_subtype`.
- Имена sync-статусов в DTO (`active/sync/idle/error`) — финально сверить с фронтовым enum при переносе.

## Следующий шаг

Перейти к скилу writing-plans — детальный план реализации (миграция → read-модель → расширение записей →
ленивый диапазон → мутации → регистрация команд → тесты).
