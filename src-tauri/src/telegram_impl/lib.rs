mod dto;
mod error;
mod live;
mod media;
mod runtime;
mod session;

pub use dto::{
    PeerDescriptor, TelegramItemContext, TelegramMessageDraft, TelegramMessageIdentity,
    ITEM_KIND_TELEGRAM_MESSAGE,
};
pub(crate) use dto::{
    TELEGRAM_PEER_KIND_CHANNEL, TELEGRAM_PEER_KIND_CHAT, TELEGRAM_PEER_KIND_USER,
};
pub use live::DialogListing;
pub use media::TelegramMediaPayload;
pub(crate) use media::{
    derive_content_kind, derive_document_media_kind, extract_item_payload, DocumentSignals,
};
#[allow(unused_imports)]
pub use runtime::{
    TelegramApiHash, TelegramClientHandle, TelegramLoginAttempt, TelegramRuntime,
    TelegramRuntimeStatus,
};
pub use session::{
    decode_session_json, encode_session_json, session_json_requires_existing_key,
    SessionEncryptionKey, TelegramSession,
};
