# Deep Dive Architecture: Extractum (v4)

## 1. Architectural Principle: "Fat Frontend, Thin Backend"
Основной принцип разработки — перенос максимального количества бизнес-логики в TypeScript (Frontend), оставляя за Rust (Backend) только задачи, требующие высокой производительности, прямого доступа к системе или работы с бинарными протоколами.

- **Rust (Backend):** Выполняет роль набора высокопроизводительных "драйверов" или "утилит". Функции Rust будут максимально простыми и атомарными (например, `get_raw_messages`, `compress_and_save`, `ask_llm_provider`).
- **TypeScript (Frontend):** Является "мозгом" приложения. Здесь происходит управление состоянием, фильтрация, агрегация данных, подготовка контекста для LLM и оркестрация вызовов к Rust-бэкенду.

## 2. LLM Provider System (Flexibility Priority)
- **Rust:** Реализует трейт `LLMProvider` и конкретные имплементации для OpenAI, Gemini, Ollama.
- **TypeScript:** Вызывает единую Rust-команду `invoke('ask_llm', ...)`, передавая в нее уже подготовленный контекст.

## 3. Data Collection (Telegram MTProto)
- **Rust (`grammers`):** Реализует полный цикл общения с MTProto, включая авторизацию, обработку ошибок (`TelegramGuard`), и запись сжатых данных в SQLite.
- **TypeScript:** Вызывает простые команды, например, `invoke('sync_channel', { channelId: ... })` и получает обратно только статус операции (успех/ошибка/прогресс).

## 4. TelegramGuard: Модуль защиты от блокировок
(Без изменений, остается полностью в зоне ответственности Rust).

## 5. Data Flow & Storage
1. **Frontend (TS):** Пользователь нажимает "Синхронизировать". Вызывается `invoke('sync_channel', ...)`.
2. **Backend (Rust):** Запускает асинхронную задачу: `grammers` -> `TelegramGuard` -> `Backpressure Channel`.
3. **Processor (Rust):** Разбирает очередь из канала -> `zstd::encode` -> `INSERT` в SQLite.
4. **Frontend (TS):):** Пользователь запрашивает сообщения. Вызывается `invoke('get_messages', ...)`.
5. **Backend (Rust):** `SELECT` -> `zstd::decode` -> Возвращает "сырые", но распакованные данные в TS.
6. **Frontend (TS):** Фильтрует, форматирует, подготавливает данные для LLM.
7. **Frontend (TS)::** Вызывает `invoke('ask_llm', ...)`.
8. **Backend (Rust):** `LLM Coordinator` -> `LLM Provider` -> Ответ возвращается в TS.

## 6. Security
(Без изменений, остается полностью в зоне ответственности Rust).
