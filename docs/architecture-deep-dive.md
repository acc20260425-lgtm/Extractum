# Deep Dive Architecture: Extractum (MVP)

## 1. Architectural Principle: "Fat Frontend, Thin Backend"

Основной принцип разработки — перенос максимального количества прикладной логики в TypeScript (Frontend), оставляя за Rust (Backend) только задачи, требующие высокой производительности, прямого доступа к системе, работы с бинарными форматами и интеграции с внешними API.

- **Rust (Backend):** Набор низкоуровневых сервисов и команд. Его зона ответственности — MTProto, работа с SQLite, сжатие/распаковка данных через ZSTD, безопасное хранение секретов, вызовы LLM-провайдеров.
- **TypeScript (Frontend):** Основной orchestration layer приложения. Здесь находятся состояние интерфейса, сценарии пользователя, фильтрация данных, подготовка SQL-параметров, сбор контекста для LLM и отображение результатов анализа.

Такой подход позволяет держать backend компактным и прикладно-нейтральным, а основную бизнес-логику и UX-логику развивать быстрее на стороне frontend.

## 2. Core Backend Responsibilities

Rust backend не должен превращаться в "второе приложение внутри приложения". Его задача — дать frontend небольшой набор надежных примитивов.

<h3>2.1 Telegram Integration</h3>
Backend реализует:
<ul>
<li>авторизацию через MTProto;</li>
<li>управление Telegram-сессией;</li>
<li>синхронизацию каналов;</li>
<li>обработку rate limits, сетевых ошибок и повторных попыток;</li>
<li>безопасное сохранение новых сообщений в SQLite.</li>
</ul>

<h3>2.2 Storage Layer</h3>
Backend отвечает за:
<ul>
<li>открытие и миграции SQLite-базы;</li>
<li>запись данных в таблицы <code>sources</code>, <code>items</code>, <code>app_settings</code>;</li>
<li>ZSTD-сжатие полей <code>content_zstd</code>, <code>raw_data_zstd</code> при записи;</li>
<li>ZSTD-распаковку при чтении;</li>
<li>выполнение параметризованных SQL-запросов для frontend-сценариев.</li>
</ul>

<h3>2.3 LLM Gateway</h3>
Backend предоставляет единый шлюз к LLM-провайдерам:
<ul>
<li>локальные провайдеры, например Ollama;</li>
<li>облачные провайдеры, например OpenAI, Gemini, Anthropic;</li>
<li>единый интерфейс вызова для frontend;</li>
<li>централизованную обработку таймаутов, ошибок и конфигурации провайдера.</li>
</ul>

<h2>3. Core Frontend Responsibilities</h2>

Frontend — это управляющий слой приложения.

Он отвечает за:
<ul>
<li>выбор источников и отображение их состояния;</li>
<li>запуск синхронизации;</li>
<li>фильтрацию данных по источнику, диапазону дат, автору и другим метаданным;</li>
<li>выбор набора записей, которые нужно показать пользователю или отправить в LLM;</li>
<li>формирование итогового контекста для LLM;</li>
<li>отображение ответа модели и связанных фрагментов данных.</li>
</ul>

Важно: frontend не должен напрямую знать детали MTProto, ZSTD или хранения секретов. Он работает только через Tauri-команды.

<h2>4. LLM Provider System</h2>

Для гибкости backend должен содержать абстракцию <code>LLMProvider</code>, скрывающую различия между локальными и облачными моделями.

Концептуально контракт выглядит так:

<ul>
<li>вход: prompt, system instructions, context blocks, provider settings;</li>
<li>выход: готовый текстовый ответ и служебные метаданные;</li>
<li>единая точка вызова из frontend: <code>invoke('ask_llm', ...)</code>.</li>
</ul>

<h3>4.1 Why no vector store</h3>
В MVP не используется векторная база и не строится embedding pipeline. Контекст для LLM формируется напрямую из данных, выбранных SQL-запросами из SQLite, после чего передается в модель как обычный текстовый контекст.

Это упрощает архитектуру, уменьшает количество moving parts и позволяет быстрее получить рабочий end-to-end сценарий анализа.

<h2>5. Storage Model</h2>

SQLite используется как единственное локальное хранилище данных.

<h3>5.1 <code>sources</code></h3>
Таблица <code>sources</code> хранит информацию об источниках данных:
<ul>
<li>тип источника;</li>
<li>внешний идентификатор;</li>
<li>отображаемое имя;</li>
<li>метаданные;</li>
<li>состояние синхронизации;</li>
<li>флаг активности.</li>
</ul>

<h3>5.2 <code>items</code></h3>
Таблица <code>items</code> хранит единицы контента:
<ul>
<li>привязку к источнику;</li>
<li>внешний ID сообщения;</li>
<li>автора;</li>
<li>дату публикации;</li>
<li>основной текст в <code>content_zstd</code>;</li>
<li>полный сырой API-ответ в <code>raw_data_zstd</code>.</li>
</ul>

<h3>5.3 <code>app_settings</code></h3>
Таблица <code>app_settings</code> используется для прикладных настроек и конфигурации.

<h3>5.4 Compression strategy</h3>
Текст сообщения и сырой JSON от Telegram сохраняются в сжатом виде через ZSTD. Это уменьшает размер локальной базы и оставляет возможность при необходимости восстановить как нормализованный текст, так и оригинальный API payload.

<h2>6. Data Flow</h2>

<h3>6.1 Channel Sync Flow</h3>
<ol>
<li>Пользователь выбирает канал и нажимает "Sync".</li>
<li>Frontend вызывает <code>invoke('sync_channel', { sourceId | channelRef })</code>.</li>
<li>Backend запускает задачу синхронизации через MTProto-клиент.</li>
<li>Полученные сообщения проходят через защитный слой TelegramGuard.</li>
<li>Processor сериализует нужные данные, сжимает тяжелые поля через ZSTD и записывает их в SQLite.</li>
</ol>

<h3>6.2 Message Retrieval Flow</h3>
<ol>
<li>Пользователь открывает источник или задает фильтры.</li>
<li>Frontend вызывает <code>invoke('get_items', filters)</code>.</li>
<li>Backend выполняет параметризованный <code>SELECT</code> по <code>source_id</code>, <code>published_at</code>, <code>author</code> и другим доступным полям.</li>
<li>Backend распаковывает <code>content_zstd</code> и, при необходимости, <code>raw_data_zstd</code>.</li>
<li>Frontend получает готовые для отображения записи.</li>
</ol>

<h3>6.3 LLM Analysis Flow</h3>
<ol>
<li>Пользователь выбирает источник, диапазон, сообщения или режим анализа.</li>
<li>Frontend либо сам отбирает нужные записи из уже загруженных данных, либо запрашивает дополнительную SQL-выборку.</li>
<li>Frontend собирает итоговый контекст: список сообщений, выдержки, метаданные, пользовательский prompt.</li>
<li>Frontend вызывает <code>invoke('ask_llm', { provider, prompt, context })</code>.</li>
<li>Backend передает запрос в <code>LLM Coordinator</code>.</li>
<li><code>LLM Coordinator</code> вызывает выбранный <code>LLMProvider</code>.</li>
<li>Ответ модели возвращается в frontend и показывается в UI.</li>
</ol>

<h2>7. TelegramGuard</h2>

<code>TelegramGuard</code> остается отдельным внутренним модулем backend и отвечает за устойчивость Telegram-интеграции.

Его задачи:
<ul>
<li>контроль частоты запросов;</li>
<li>retry/backoff;</li>
<li>нормализация ошибок;</li>
<li>защита от "слишком агрессивной" синхронизации;</li>
<li>предсказуемое поведение фоновых задач синка.</li>
</ul>

Этот модуль не должен протекать в frontend API. Для frontend всё должно выглядеть как простой статус синхронизации: <code>idle</code>, <code>running</code>, <code>completed</code>, <code>failed</code>.

<h2>8. Security Boundaries</h2>

Безопасность должна быть сконцентрирована в backend.

Backend отвечает за:
<ul>
<li>хранение API-ключей через системный keyring;</li>
<li>хранение Telegram-сессии в app data directory;</li>
<li>недопущение утечки секретов в логах;</li>
<li>валидацию входных параметров Tauri-команд;</li>
<li>ограничение произвольного выполнения SQL извне.</li>
</ul>

Frontend не должен иметь прямого доступа ни к ключам, ни к Telegram session storage.

<h2>9. Recommended Tauri Commands for MVP</h2>

Для MVP достаточно небольшого и понятного API между frontend и backend:

<ul>
<li><code>init_database</code></li>
<li><code>list_sources</code></li>
<li><code>add_telegram_source</code></li>
<li><code>sync_channel</code></li>
<li><code>get_items</code></li>
<li><code>get_item_by_id</code></li>
<li><code>get_sync_status</code></li>
<li><code>ask_llm</code></li>
<li><code>get_settings</code></li>
<li><code>save_settings</code>.</li>
</ul>

Важно держать команды прикладными и ограниченными по ответственности. Не стоит делать одну универсальную команду "execute_anything".

<h2>10. MVP Boundary</h2>

В MVP входят:
<ul>
<li>подключение Telegram через MTProto;</li>
<li>хранение данных в SQLite;</li>
<li>просмотр источников и сообщений;</li>
<li>фильтрация по базовым метаданным;</li>
<li>отправка SQL-выбранного контекста в LLM;</li>
<li>поддержка как минимум одного LLM-провайдера.</li>
</ul>

В MVP не входят:
<ul>
<li>vector store;</li>
<li>embeddings;</li>
<li>semantic search;</li>
<li>автоматический retrieval pipeline;</li>
<li>multi-source ingestion beyond Telegram.</li>
</ul>

<h2>11. Next Steps</h2>

<ol>
<li>Зафиксировать SQLite schema как единственный источник правды для storage-модели.</li>
<li>Привести <code>design-document.md</code> в соответствие с этой архитектурой.</li>
<li>Удалить из <code>database-schema.md</code> поле <code>is_embedded</code> и индекс <code>idx_items_embedded</code>, так как векторизация в MVP отсутствует.</li>
<li>Реализовать минимальный набор Tauri-команд для sync, select и ask_llm.</li>
<li>Построить простой UI-поток: source list -&gt; message list -&gt; ask LLM.</li>
</ol>