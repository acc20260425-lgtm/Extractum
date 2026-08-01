mod dto;
mod error;
mod live;
mod media;
mod runtime;
mod session;
mod takeout;

pub use dto::{
    ForumTopicSnapshot, PeerDescriptor, TelegramItemContext, TelegramMessageDraft,
    TelegramMessageIdentity, ITEM_KIND_TELEGRAM_MESSAGE,
};
pub use live::{DialogListing, LiveMessage, LiveMessageBatch};
pub use media::TelegramMediaPayload;
#[allow(unused_imports)]
pub use runtime::{
    TelegramApiHash, TelegramClientHandle, TelegramLoginAttempt, TelegramRuntime,
    TelegramRuntimeStatus,
};
pub use session::{
    decode_session_json, encode_session_json, session_json_requires_existing_key,
    SessionEncryptionKey, TelegramSession,
};
#[cfg(test)]
pub(crate) use takeout::attempt_fixture as takeout_attempt_fixture;
#[cfg(test)]
pub(crate) use takeout::fallback_fixture as takeout_fallback_fixture;
pub use takeout::{
    MessageRange, TakeoutAttempt, TakeoutCount, TakeoutFallback, TakeoutFallbackKind,
    TakeoutMessage, TakeoutPage, TakeoutPeer, TakeoutTransport,
};
