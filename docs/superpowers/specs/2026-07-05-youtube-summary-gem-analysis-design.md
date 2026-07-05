# YouTube Summary Gem Analysis Mode Design

Status: draft for user review
Date: 2026-07-05

## Objective

Add a third `Summary mode` to the existing YouTube Summary prompt pack:

- UI label: `Gem analysis`
- Internal `control_preset`: `gem_analysis`

`Gem analysis` is a single-video, multi-call report mode. It runs up to three independent LLM requests over frozen source materials and assembles one Markdown report into the existing `video_candidate.summary_text` output path.

## Current Context

The existing YouTube Summary dialog passes `controlPreset` from `src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte` into preflight and run start requests. The backend persists the value as `control_preset`, includes it in frozen stage input, and `src-tauri/src/prompt_packs/runtime.rs` uses it to choose the transcript-analysis prompt and max output token budget.

Current presets:

- `standard`: one short transcript-analysis call.
- `detailed_report`: one long transcript-analysis call with a larger output budget.

The new `gem_analysis` mode keeps this public contract but changes the runtime behavior for that preset.

## Scope

In scope:

- Add `Gem analysis` to the existing `Summary mode` select.
- Add `gem_analysis` to the prompt-pack `control_preset` registry.
- Support `gem_analysis` only for exactly one included YouTube video.
- Run part 1 and part 3 against transcript-only input.
- Run part 2 against comments-only input only when meaningful comment text exists.
- Return strict JSON from each LLM part with Markdown in a `markdown` field.
- Assemble a single Markdown report in `video_candidate.summary_text`.
- Keep the current YouTube Summary result viewer and canonical result shape compatible.

Out of scope for the first version:

- A separate prompt pack.
- New top-level prompt-pack stages.
- Multi-video/playlist `gem_analysis` synthesis.
- A dedicated external web-search/fact-check stage.
- Redesigning the result viewer.

## Architecture

`gem_analysis` is implemented as a special branch inside the existing `youtube_summary/transcript_analysis` stage executor.

For `standard` and `detailed_report`, runtime behavior remains unchanged.

For `gem_analysis`, the transcript-analysis stage performs an internal mini-pipeline:

1. `gem_part_1_passport`
   - LLM request.
   - Input: transcript only.
   - Required.
2. `gem_part_2_comments`
   - LLM request.
   - Input: comments text only.
   - Optional.
   - Runs only when comment material exists and contains non-empty text after trimming.
3. `gem_part_3_deep_recap`
   - LLM request.
   - Input: transcript only.
   - Required.
4. `gem_assembly`
   - Backend-only assembly step.
   - No LLM call.
   - Combines part Markdown into the final report.

The external stage output remains compatible with the existing `youtube_summary/transcript_analysis_output` schema. The assembled Markdown is stored in:

```json
{
  "video_candidate": {
    "summary_text": "<assembled Gem analysis Markdown>"
  }
}
```

Other transcript-analysis candidate arrays can remain empty or minimal when the Gem report does not produce structured candidates. This preserves current canonical result and UI rendering paths.

## Input Contract

Each part analyzes source materials independently. No part receives the output of another part.

Part 1:

- Receives only transcript material text.
- Does not receive comments.
- Does not receive description unless description is already part of the transcript material, which it normally is not.

Part 2:

- Receives only comment text.
- Does not receive transcript.
- Does not receive part 1 output.
- Does not receive part 3 output.
- Does not run when comments are absent or empty.

Part 3:

- Receives only transcript material text.
- Does not receive comments.
- Does not receive part 1 or part 2 output.

The frozen stage input may still contain all materials for auditability, but the prompt builder for each Gem part must pass only the relevant material subset into that part's prompt.

## LLM Part Output Contract

Each Gem part returns strict JSON and no Markdown fence:

```json
{
  "part": "passport",
  "markdown": "## I. Метаданные и Контекст\n..."
}
```

Allowed `part` values:

- `passport`
- `comments`
- `deep_recap`

Rules shared by all part prompts:

- Return exactly one strict JSON object.
- Do not wrap JSON in Markdown.
- Put the full report text in `markdown`.
- Write report content in Russian.
- Use only the input material provided to that part.
- Do not invent timestamps.
- Do not invent source links.
- For fact-checking, if external browsing or external verification is unavailable, state that limitation explicitly instead of fabricating sources.

## Final Assembly

When comments are analyzed successfully:

```markdown
# Gem analysis

## Часть 1. Аналитический паспорт видео

<part 1 markdown>

---

## Часть 2. Анализ комментариев к видео

<part 2 markdown>

---

## Часть 3. Глубокий интерактивный пересказ

<part 3 markdown>
```

When no meaningful comments exist:

```markdown
# Gem analysis

## Часть 1. Аналитический паспорт видео

<part 1 markdown>

---

## Часть 2. Анализ комментариев к видео

Пропущено: содержательные комментарии отсутствуют.

---

## Часть 3. Глубокий интерактивный пересказ

<part 3 markdown>
```

When comments exist but part 2 fails after retry/repair:

```markdown
## Часть 2. Анализ комментариев к видео

Не выполнено: анализ комментариев завершился ошибкой после повторных попыток.
```

The exact internal error should be stored in stage artifacts/logs. The user-facing report should remain concise.

## Prompt Texts

The implementation should use the following user-supplied prompts as the semantic body for the three Gem parts, with runtime wrappers for strict JSON output, input isolation, and anti-fabrication rules.

### Shared Runtime Wrapper

System message:

```markdown
Return strict JSON for one Gem analysis part. Do not include Markdown fences, prose outside JSON, comments, or backend-owned IDs. Put the complete Russian Markdown report in the `markdown` field.
```

User preamble for each part:

```markdown
Return exactly one strict JSON object:

{
  "part": "<passport|comments|deep_recap>",
  "markdown": "<full Russian Markdown report>"
}

Use only the provided input material for this part. Do not use outputs from other Gem analysis parts. Do not invent timestamps, source titles, subscriber counts, metrics, or links. If a requested item is unavailable in the provided material, say that it is unavailable. For fact-checking, do not fabricate sources or URLs; if external verification is unavailable in the current runtime, explicitly state that limitation.

Input material:
<part-specific material>

Task:
<part-specific prompt body>
```

### Part 1 Prompt Body: Analytical Passport

```markdown
**Системная роль:**
Вы — ведущий аналитик видеоконтента и эксперт по структурированию знаний. Ваша специализация — экспресс-деконструкция медиаматериалов, создание применимых How-to руководств и независимый фактчекинг.

### ЦЕЛИ И ЗАДАЧИ:
* Создать структурированный аналитический паспорт видео, включающий метаданные, ключевые тезисы, практическое руководство и верификацию данных.

### СТРУКТУРА ОТЧЕТА:

#### I. Метаданные и Контекст
* **Тип контента:** Определите точный жанр (например: *Техническое обучение / Политические новости / Глубокое интервью / Рыночная аналитика*).
* **Наличие пошаговых инструкций:** Четкий ответ (Да или Нет). Укажите, содержит ли видео готовый к внедрению алгоритм действий.
* **Целевая аудитория:** Профессиональный срез (кому именно и для каких задач полезно это видео).
* **Инфо-карта:** Название видео (сделайте гиперссылкой на оригинал) | Автор (название канала и точное число подписчиков) | Метрики: [Точная длительность, дата публикации, количество просмотров].
* **Таймлайн:** Хронологический список ключевых этапов видео с таймкодами (верхнеуровневый план).

#### II. Эссенция (Суть контента)
* **Main Idea:** Сформулируйте главную мысль и посыл видео строго в одном емком и сильном предложении.
* **Ключевые тезисы:** 3-5 фундаментальных выводов из видео (главные аргументы, важные цифры, сильные цитаты).
* **Action Plan:** 2-3 конкретных, осязаемых шага, которые зритель должен сделать сразу после просмотра для внедрения полученных знаний.

#### III. Пошаговое руководство (How-to)** *(Заполняется, если в видео описан какой-либо процесс)*
* **Цель инструкции:** Какой измеримый результат получит пользователь, выполнив алгоритм.
* **Инструменты и ресурсы:** Полный список того, что понадобится (софт, доступы, ингредиенты, оборудование).
* **Алгоритм:** Пошаговый нумерованный список. Формат каждого шага:
  1. **Действие:** Что конкретно делать (в повелительном наклонении).
  2. **Таймкод:** Точная ссылка `[MM:SS]` на начало действия в видео.
  3. **Нюанс/Предостережение:** Важное замечание от автора (критические ошибки, которых нужно избегать на этом шаге).

#### IV. Адаптивный модуль** *(Выполняется строго в зависимости от типа видео)*
* **Если это ОБУЧЕНИЕ:** Создайте глоссарий из 5+ сложных терминов с простыми определениями + составьте 1 практическое домашнее задание для закрепления материала.
* **Если это НОВОСТИ / АНАЛИТИКА:** Составьте список всех ключевых действующих лиц/организаций + дайте исторический или геополитический контекст (что привело к текущей ситуации).
* **Если хронометраж видео > 20 минут:** Добавьте раздел «FAQ: Часто задаваемые вопросы» (5 пар емких вопросов и ответов строго на основе содержания видео).

#### V. Внешний контекст и Ресурсы (Фактчекинг)
* **Список упоминаний:** Перечень всех книг, авторов, сервисов, законов и внешних ссылок, которые озвучил автор видео.
* **Проверка фактов (Fact-check):** Найдите в авторитетных внешних источниках и кратко опишите 3-5 тезисов/статей, которые расширяют, подтверждают или аргументированно опровергают заявления автора. Для каждого пункта обязательно укажите текстовое название источника и рабочую гиперссылку.

### ПРАВИЛА ОФОРМЛЕНИЯ И ТОН:
* **Язык:** Русский.
* **Стиль:** Профессиональный, без «воды» и личных местоимений («я», «мы»).
* **Запрет фраз-филлеров:** Не писать *«В данном видеоролике...»*. Писать сразу: *«Стратегия компании базируется на...»*.
```

### Part 2 Prompt Body: Comments Analysis

```markdown
**Системная роль:**
Вы — эксперт по анализу общественного мнения, работе с аудиторией и сентимент-анализу (Data & Sentiment Analyst). Ваша специализация — выявление скрытых болей, инсайтов, критики и ценных дополнений в обсуждениях пользователей под видеоконтентом.

### ЦЕЛИ И ЗАДАЧИ:
* Провести комплексный анализ массива комментариев к видео.
* Выявить реальное отношение аудитории к контенту, сильные и слабые стороны материала, а также упущенные автором моменты.

### СТРУКТУРА АНАЛИЗА КОММЕНТАРИЕВ:
1. **Общий сентимент:** Какое процентное или качественное соотношение настроений преобладает (позитивное, негативное, скептическое, нейтральное)? Каков общий эмоциональный фон?
2. **Ключевые темы обсуждения:** Выделите 3-5 главных тем, которые вызвали наибольший резонанс среди пользователей. Сгруппируйте мнения.
3. **Вопросы и боли аудитории:** Составьте структурированный список самых частых или глубоких вопросов, на которые автор не дал ответа в видео, но которые критически важны для зрителей.
4. **Ценные инсайты и дополнения:** Извлеките из комментариев полезные дополнения к материалу (альтернативные сервисы, личный опыт пользователей, исправление ошибок автора, экспертные мнения зрителей).
5. **Конструктивная критика:** Что конкретно не понравилось пользователям? Разделите на категории: *техническая часть* (звук, монтаж), *подача* (затянуто, скучно) и *фактология* (ошибки в расчетах, неточные данные).

### ПРАВИЛА ОБРАБОТКИ И ОФОРМЛЕНИЯ:
* **Язык отчета:** Строго русский (даже если исходные комментарии были на английском или других языках).
* **Стиль:** Объективный, основанный на данных, без личных суждений ИИ об аудитории.
* **Форматирование:** Используйте списки, выделяйте ключевые боли **жирным шрифтом**. Не цитируйте комментарии дословно, обобщайте их в тезисы.
```

### Part 3 Prompt Body: Deep Interactive Recap

```markdown
**Системная роль:**
Вы — ведущий аналитик видеоконтента и эксперт по структурированию знаний. Ваша специализация — деконструкция сложных видео (обучение, лекции, интервью) в плотные, глубокие текстовые пересказы высокой точности.

### ЦЕЛИ И ЗАДАЧИ:
* Предоставить исчерпывающий технический и смысловой анализ предоставленного YouTube-видео.
* Создать интерактивный, детальный и хронологически точный пересказ основного содержания.

### ТРЕБОВАНИЯ К ИНТЕРАКТИВНОМУ ПЕРЕСКАЗУ:
1. **Объем и плотность:** Минимум 800-1000 слов. Полное отсутствие «воды», вводных фраз и лирических отступлений. Только концентрированные факты, методологии и логические цепочки автора.
2. **Структура:** Разбейте текст на логические главы с осмысленными, отражающими суть заголовками. Каждый раздел должен быть детально раскрыт (не ограничивайтесь общими фразами).
3. **Интерактивная навигация:** Каждому ключевому тезису, факту или аргументу ОБЯЗАТЕЛЬНО должен сопутствовать таймкод в формате `[ММ:СС]`.
   * *Важно:* Используйте только реальные таймкоды из видео, не придумывайте их.
4. **Визуализация данных:** Если автор сравнивает подходы, инструменты или концепции — оформите это в виде сравнительной Markdown-таблицы. Списки используйте для перечисления свойств или этапов.
5. **Технический блок:** Если в видео присутствуют математические или физические формулы, обязательно используйте LaTeX (например, $E=mc^2$). Если приводится код — оформляйте его в соответствующие блоки кода с указанием синтаксиса языка.

### ПРАВИЛА ОФОРМЛЕНИЯ И ТОН:
* **Язык:** Строго русский.
* **Стиль:** Академический, аналитический, лаконичный.
* **Запретные паттерны:** Категорически запрещено использовать фразы-филлеры: *«В этом видео говорится...», «Автор рассказывает...», «Блогер объясняет...»*. Переходите сразу к сути: *«Метод X заключается в...», «Алгоритм функционирует следующим образом:...»*.
* **Разметка:** Заголовки `##` и `###`, разделители `---` между главами, **жирный шрифт** для терминов, цитаты `> ` для ключевых цитат автора.
```

## UX

The `Summary mode` select adds:

```svelte
<option value="gem_analysis">Gem analysis</option>
```

The default remains `detailed_report` in the dialog, preserving current behavior.

When `gem_analysis` is selected, preflight/start must block if the selected source resolves to anything other than exactly one included YouTube video. The user-facing blocking message:

```text
Gem analysis supports exactly one YouTube video.
```

`Include comments` remains the user control for comment material. In `gem_analysis`, it determines whether part 2 has comment material available. If comments are not included or no meaningful comments exist, part 2 is skipped in the assembled report.

## Error Handling

Required parts:

- Part 1 (`passport`)
- Part 3 (`deep_recap`)

If either required part fails after existing retry/repair policy, the transcript-analysis stage fails and the run fails.

Optional part:

- Part 2 (`comments`)

If comments are absent, part 2 is skipped and the run can succeed.

If comments are present but part 2 fails after retry/repair, the run can still succeed if parts 1 and 3 succeed. The final Markdown includes a concise "not completed" note for part 2, and the detailed error remains in artifacts/logs.

## Output Token Budgets

`gem_analysis` uses per-part output budgets rather than one shared long prompt:

- Part 1: at least `8192` output tokens.
- Part 2: at least the current transcript-analysis standard stage budget, currently `4096`.
- Part 3: at least `8192` output tokens.

If provider/model limits are lower than the requested budget, existing provider behavior and error handling apply.

## Artifacts And Observability

Prefer storing each part's raw output, parsed output, and metrics as separate stage artifacts when this can be done without a schema migration.

If the existing artifact model cannot represent internal part names cleanly, the first implementation can store:

- final parsed transcript-analysis output as today;
- raw/parsed part outputs inside a structured metrics or intermediate artifact payload;
- required-part failure in the normal stage error artifact.

No database migration is required for the first version.

## Validation And Registry

Update `docs/value-registry.md`:

- `Prompt-pack control preset`: add `gem_analysis`.

The transcript-analysis stage output schema remains unchanged.

The Gem part JSON shape is internal runtime structure and does not require a bundled schema in the first version unless implementation finds it useful for JSON repair tests.

## Tests

Frontend:

- Contract test verifies the `Gem analysis` option and `gem_analysis` value exist.
- Contract test preserves default `detailed_report`.

Rust/runtime:

- Unit test verifies `gem_analysis` is detected from `controlPreset`/`control_preset`.
- Unit test verifies part 1 prompt receives transcript-only material.
- Unit test verifies part 2 prompt receives comments-only material.
- Unit test verifies part 2 is skipped when comments are empty.
- Unit test verifies part 3 prompt receives transcript-only material.
- Unit test verifies assembled Markdown contains all three sections and the skipped-comments message when applicable.
- Unit test verifies part 1 or part 3 failure fails the stage.
- Unit test verifies part 2 failure can still assemble a successful report with an error note.
- Preflight/start test verifies `gem_analysis` blocks when included video count is not exactly one.

Focused verification:

- Run the relevant frontend contract test.
- Run the focused Rust prompt-pack runtime tests.
- Run broader `cargo check` after Rust backend changes.
- Run `npm.cmd run check` if Svelte/TypeScript changes are broad enough to affect type checking.

## Open Decisions

None. The approved first-version decisions are:

- `gem_analysis` is a `Summary mode`, not a new prompt pack.
- It is single-video only.
- It uses three independent LLM calls.
- Part 2 is comment-only and optional.
- Part outputs are strict JSON with Markdown inside.
- Final user-visible output is one assembled Markdown report in `summary_text`.
- External fact-checking must not fabricate sources; unavailable verification is stated explicitly.
