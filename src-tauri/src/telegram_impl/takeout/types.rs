#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TakeoutFallbackKind {
    ExportDc,
    OnlyMyMessages,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TakeoutAttempt {
    home_dc_id: i32,
    export_dc_id: i32,
}

impl TakeoutAttempt {
    pub(super) fn new(home_dc_id: i32, export_dc_id: i32) -> Self {
        Self {
            home_dc_id,
            export_dc_id,
        }
    }

    pub fn home_dc_id(&self) -> i32 {
        self.home_dc_id
    }

    pub fn export_dc_id(&self) -> i32 {
        self.export_dc_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TakeoutFallback {
    kind: TakeoutFallbackKind,
    warning: String,
    provenance_message: Option<String>,
}

impl TakeoutFallback {
    pub(super) fn new(
        kind: TakeoutFallbackKind,
        warning: String,
        provenance_message: Option<String>,
    ) -> Self {
        Self {
            kind,
            warning,
            provenance_message,
        }
    }

    pub fn kind(&self) -> TakeoutFallbackKind {
        self.kind
    }

    pub fn warning(&self) -> &str {
        &self.warning
    }

    pub fn provenance_message(&self) -> Option<&str> {
        self.provenance_message.as_deref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TakeoutPeer {
    source_subtype: String,
    peer_kind: &'static str,
    peer_id: i64,
    access_hash: Option<i64>,
}

impl TakeoutPeer {
    pub fn from_descriptor(descriptor: &PeerDescriptor) -> AppResult<Self> {
        let peer_id = descriptor
            .external_id
            .parse::<i64>()
            .ok()
            .filter(|peer_id| *peer_id > 0)
            .ok_or_else(|| {
                AppError::validation(format!(
                    "Invalid Telegram peer id '{}'",
                    descriptor.external_id
                ))
            })?;

        match descriptor.source_subtype.as_str() {
            "channel" | "supergroup" => {
                let access_hash = descriptor.access_hash.ok_or_else(|| {
                    AppError::validation(format!(
                        "Telegram {} {} is missing an access hash",
                        descriptor.source_subtype, descriptor.external_id
                    ))
                })?;
                PeerId::channel(peer_id).ok_or_else(|| {
                    AppError::validation(format!("Invalid Telegram channel peer id {peer_id}"))
                })?;
                Ok(Self::new(
                    descriptor.source_subtype.clone(),
                    "channel",
                    peer_id,
                    Some(access_hash),
                ))
            }
            "group" => {
                PeerId::chat(peer_id).ok_or_else(|| {
                    AppError::validation(format!("Invalid Telegram group peer id {peer_id}"))
                })?;
                Ok(Self::new(
                    descriptor.source_subtype.clone(),
                    "chat",
                    peer_id,
                    None,
                ))
            }
            other => Err(AppError::validation(format!(
                "Unsupported Telegram source_subtype '{other}'"
            ))),
        }
    }

    pub(super) fn new(
        source_subtype: String,
        peer_kind: &'static str,
        peer_id: i64,
        access_hash: Option<i64>,
    ) -> Self {
        Self {
            source_subtype,
            peer_kind,
            peer_id,
            access_hash,
        }
    }

    pub fn peer_kind(&self) -> &'static str {
        self.peer_kind
    }

    pub fn peer_id(&self) -> i64 {
        self.peer_id
    }

    pub(super) fn source_subtype(&self) -> &str {
        &self.source_subtype
    }

    pub(super) fn access_hash(&self) -> Option<i64> {
        self.access_hash
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageRange {
    min_id: i32,
    max_id: i32,
}

impl MessageRange {
    pub(super) fn new(min_id: i32, max_id: i32) -> Self {
        Self { min_id, max_id }
    }

    pub fn min_id(&self) -> i32 {
        self.min_id
    }

    pub fn max_id(&self) -> i32 {
        self.max_id
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TakeoutCount {
    count: i64,
    only_my_messages: bool,
}

impl TakeoutCount {
    pub(super) fn new(count: i64, only_my_messages: bool) -> Self {
        Self {
            count,
            only_my_messages,
        }
    }

    pub fn count(&self) -> i64 {
        self.count
    }

    pub fn only_my_messages(&self) -> bool {
        self.only_my_messages
    }
}

#[derive(Debug)]
pub struct TakeoutPage {
    profile: TakeoutPaginationProfile,
    cursor: TakeoutPaginationCursor,
    page_index: usize,
    messages: Vec<TakeoutMessage>,
    has_next: bool,
    only_my_messages: bool,
    pagination_fallback_warning: Option<String>,
}

impl TakeoutPage {
    pub(super) fn from_parts(
        profile: TakeoutPaginationProfile,
        cursor: TakeoutPaginationCursor,
        page_index: usize,
        messages: Vec<TakeoutMessage>,
        has_next: bool,
        only_my_messages: bool,
        pagination_fallback_warning: Option<String>,
    ) -> Self {
        Self {
            profile,
            cursor,
            page_index,
            messages,
            has_next,
            only_my_messages,
            pagination_fallback_warning,
        }
    }

    pub fn take_messages(&mut self) -> Vec<TakeoutMessage> {
        std::mem::take(&mut self.messages)
    }

    pub fn has_next(&self) -> bool {
        self.has_next
    }

    pub fn only_my_messages(&self) -> bool {
        self.only_my_messages
    }

    pub fn take_pagination_fallback_warning(&mut self) -> Option<String> {
        self.pagination_fallback_warning.take()
    }

    pub(super) fn pagination_state(
        &self,
    ) -> (TakeoutPaginationProfile, TakeoutPaginationCursor, usize) {
        (self.profile, self.cursor, self.page_index)
    }
}

#[derive(Debug)]
pub struct TakeoutMessage {
    raw: tl::types::Message,
}

impl TakeoutMessage {
    pub(super) fn from_raw(raw: tl::types::Message) -> Self {
        Self { raw }
    }

    pub fn message_id(&self) -> i64 {
        i64::from(self.raw.id)
    }

    pub fn into_draft(self, source_title: Option<&str>) -> AppResult<Option<TelegramMessageDraft>> {
        let source_title = source_title.map(str::to_string);
        super::raw_parse::parse_raw_message(&source_title, self.raw).map_err(AppError::internal)
    }
}
use extractum_core::error::{AppError, AppResult};
use grammers_client::tl;
use grammers_session::types::PeerId;

use super::super::dto::{PeerDescriptor, TelegramMessageDraft};
use super::pagination::{TakeoutPaginationCursor, TakeoutPaginationProfile};
