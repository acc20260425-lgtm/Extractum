# Design Document: Extractum

## 1. Project Overview
**Extractum** is a desktop application (Windows/macOS/Linux) designed to collect, store, and analyze information from diverse digital sources, starting with Telegram channels. It leverages local and cloud LLMs to provide deep insights and semantic search across the collected data.

## 2. Core Functional Requirements
### 2.1 Data Collection (Module 1)
- **Source:** Telegram Channels (Public and Subscribed).
- **Mechanism:** MTProto (User Client API) for maximum data access.
- **Background Tasks:** Periodic syncing and real-time monitoring of selected channels.
- **Storage:** Metadata and raw message content in SQLite.

### 2.2 Data Analysis (Module 2)
- **Embeddings:** Automatic generation of vector embeddings for all collected text.
- **Semantic Search:** Fast retrieval of relevant context based on user queries.
- **LLM Integration:**
    - **Local:** Ollama/llama.cpp support for private, offline analysis.
    - **Cloud:** Integration with OpenAI, Anthropic, and Gemini APIs.
- **RAG Pipeline:** Retrieval-Augmented Generation to answer questions based on the local knowledge base.

## 3. Tech Stack
- **Backend (Tauri/Rust):**
    - `tauri-plugin-sql`: For SQLite management.
    - `grammers` or `tdlib-sys`: Rust crates for MTProto communication.
    - `lancedb`: Embeddable vector database for Rust.
- **Frontend (SvelteKit/TypeScript):**
    - `shadcn-svelte` or `Skeleton UI`: For high-quality UI components.
    - `Lucide Icons`: For iconography.
- **Communication:** Tauri commands (IPC) between frontend and backend.

## 4. Architectural Components
### 4.1 Backend Services (Rust)
1. **Telegram Manager:** Handles authentication, session management, and message fetching.
2. **Database Manager:** Manages SQLite schema and migrations.
3. **Vector Manager:** Handles indexing and semantic retrieval.
4. **LLM Coordinator:** Bridges local/cloud LLM requests and manages prompt templates.

### 4.2 Frontend Layers (SvelteKit)
1. **Dashboard:** Overview of synced sources and latest data.
2. **Source Manager:** UI for adding/removing Telegram channels and monitoring sync status.
3. **Analysis Lab:** Chat interface for querying data and generating reports.
4. **Settings:** API key management and local LLM configuration.

## 5. Data Schema (Draft)
- **Channels:** `id`, `name`, `username`, `description`, `sync_active`, `last_sync_id`.
- **Messages:** `id`, `channel_id`, `text`, `timestamp`, `metadata_json`, `embedding_indexed`.
- **Vector Index:** Optimized for fast cosine similarity search on message embeddings.

## 6. Implementation Phases
1. **Phase 1: Foundations** - Setup Tauri, SQLite, and basic Svelte UI.
2. **Phase 2: Telegram Client** - Implement MTProto auth and basic message fetching.
3. **Phase 3: Vector Storage** - Integrate LanceDB and embedding generation (local/cloud).
4. **Phase 4: LLM Lab** - Build the RAG pipeline and analysis interface.
5. **Phase 5: Polish** - Refine UX, add multi-source support, and documentation.
