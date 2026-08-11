use grammers_client::tl;

use extractum_core::error::{AppError, AppResult};

use super::types::{MessageRange, TakeoutMessage, TakeoutPage};

const TAKEOUT_HISTORY_PAGE_LIMIT: i32 = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TakeoutPaginationProfile {
    TDesktop,
    DescendingFallback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TakeoutPageRequest {
    pub(super) offset_id: i32,
    pub(super) add_offset: i32,
    pub(super) limit: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TakeoutPaginationCursor {
    TDesktop { largest_id_plus_one: i32 },
    DescendingFallback { offset_id: i32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TakeoutCursorAdvance {
    pub(super) cursor: TakeoutPaginationCursor,
    pub(super) advanced: bool,
    pub(super) reached_range_start: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TakeoutPaginationFallbackReason {
    EmptyFirstPageWithNonZeroCount,
    NonAdvancingTDesktopCursor,
}

pub(super) fn select_history_splits(
    telegram_source_subtype: &str,
    split_ranges: Vec<tl::enums::MessageRange>,
) -> AppResult<Vec<MessageRange>> {
    let mut ranges = if split_ranges.is_empty() {
        vec![fallback_message_range()]
    } else {
        split_ranges
            .into_iter()
            .map(|range| match range {
                tl::enums::MessageRange::Range(range) => {
                    MessageRange::new(range.min_id, range.max_id)
                }
            })
            .collect()
    };

    match telegram_source_subtype {
        "channel" | "supergroup" => Ok(vec![ranges.pop().unwrap_or_else(fallback_message_range)]),
        "group" => Ok(ranges),
        other => Err(AppError::validation(format!(
            "Unsupported Telegram source_subtype '{other}'"
        ))),
    }
}

fn fallback_message_range() -> MessageRange {
    MessageRange::new(1, i32::MAX)
}

impl TakeoutPaginationCursor {
    pub(super) fn new(profile: TakeoutPaginationProfile, range: &MessageRange) -> Self {
        match profile {
            TakeoutPaginationProfile::TDesktop => Self::TDesktop {
                largest_id_plus_one: 1,
            },
            TakeoutPaginationProfile::DescendingFallback => Self::DescendingFallback {
                offset_id: range.max_id(),
            },
        }
    }
}

pub(super) fn takeout_page_request(cursor: TakeoutPaginationCursor) -> TakeoutPageRequest {
    match cursor {
        TakeoutPaginationCursor::TDesktop {
            largest_id_plus_one,
        } => TakeoutPageRequest {
            offset_id: largest_id_plus_one,
            add_offset: -TAKEOUT_HISTORY_PAGE_LIMIT,
            limit: TAKEOUT_HISTORY_PAGE_LIMIT,
        },
        TakeoutPaginationCursor::DescendingFallback { offset_id } => TakeoutPageRequest {
            offset_id,
            add_offset: 0,
            limit: TAKEOUT_HISTORY_PAGE_LIMIT,
        },
    }
}

pub(super) fn next_takeout_cursor(
    cursor: TakeoutPaginationCursor,
    message_ids: &[i32],
    range: &MessageRange,
) -> TakeoutCursorAdvance {
    let min_id = range.min_id();
    match cursor {
        TakeoutPaginationCursor::TDesktop {
            largest_id_plus_one,
        } => {
            let next_largest_id_plus_one = message_ids
                .iter()
                .copied()
                .filter(|message_id| *message_id > min_id)
                .max()
                .map(|message_id| message_id.saturating_add(1))
                .unwrap_or(largest_id_plus_one);
            TakeoutCursorAdvance {
                cursor: TakeoutPaginationCursor::TDesktop {
                    largest_id_plus_one: next_largest_id_plus_one,
                },
                advanced: next_largest_id_plus_one > largest_id_plus_one,
                reached_range_start: false,
            }
        }
        TakeoutPaginationCursor::DescendingFallback { offset_id } => {
            let next_offset_id = message_ids
                .iter()
                .copied()
                .filter(|message_id| *message_id > min_id)
                .fold(offset_id, i32::min);
            TakeoutCursorAdvance {
                cursor: TakeoutPaginationCursor::DescendingFallback {
                    offset_id: next_offset_id,
                },
                advanced: next_offset_id < offset_id,
                reached_range_start: next_offset_id <= min_id,
            }
        }
    }
}

pub(super) fn should_restart_with_descending_fallback(
    profile: TakeoutPaginationProfile,
    split_count: i64,
    page_index: usize,
    message_count: usize,
    advance: TakeoutCursorAdvance,
) -> Option<TakeoutPaginationFallbackReason> {
    if profile != TakeoutPaginationProfile::TDesktop {
        return None;
    }
    if page_index == 0 && split_count > 0 && message_count == 0 {
        return Some(TakeoutPaginationFallbackReason::EmptyFirstPageWithNonZeroCount);
    }
    if message_count > 0 && !advance.advanced {
        return Some(TakeoutPaginationFallbackReason::NonAdvancingTDesktopCursor);
    }
    None
}

pub(super) fn takeout_pagination_fallback_warning(
    reason: TakeoutPaginationFallbackReason,
    range: &MessageRange,
) -> String {
    let reason = match reason {
        TakeoutPaginationFallbackReason::EmptyFirstPageWithNonZeroCount => "an empty first page",
        TakeoutPaginationFallbackReason::NonAdvancingTDesktopCursor => "a non-advancing cursor",
    };
    format!(
        "TDesktop Takeout pagination returned {reason} for split {}..{}; retrying this split with Extractum descending fallback.",
        range.min_id(),
        range.max_id()
    )
}

pub(super) fn parse_takeout_page(
    response: tl::enums::messages::Messages,
    profile: TakeoutPaginationProfile,
    cursor: TakeoutPaginationCursor,
    range: &MessageRange,
    split_count: i64,
    page_index: usize,
    only_my_messages: bool,
) -> AppResult<TakeoutPage> {
    let (messages, is_terminal_response) = match response {
        tl::enums::messages::Messages::Messages(messages) => (messages.messages, true),
        tl::enums::messages::Messages::Slice(messages) => (messages.messages, false),
        tl::enums::messages::Messages::ChannelMessages(messages) => (messages.messages, false),
        tl::enums::messages::Messages::NotModified(_) => {
            return Err(AppError::network(
                "Telegram returned messagesNotModified for Takeout history page",
            ));
        }
    };

    let mut messages = messages
        .into_iter()
        .filter_map(|message| match message {
            tl::enums::Message::Message(message) => Some(message),
            _ => None,
        })
        .collect::<Vec<_>>();
    if profile == TakeoutPaginationProfile::TDesktop {
        messages.reverse();
    }

    let message_ids = messages
        .iter()
        .map(|message| message.id)
        .collect::<Vec<_>>();
    let advance = next_takeout_cursor(cursor, &message_ids, range);
    if let Some(reason) = should_restart_with_descending_fallback(
        profile,
        split_count,
        page_index,
        messages.len(),
        advance,
    ) {
        return Ok(TakeoutPage::from_parts(
            TakeoutPaginationProfile::DescendingFallback,
            TakeoutPaginationCursor::new(TakeoutPaginationProfile::DescendingFallback, range),
            0,
            Vec::new(),
            true,
            only_my_messages,
            Some(takeout_pagination_fallback_warning(reason, range)),
        ));
    }

    let has_next = !messages.is_empty()
        && !is_terminal_response
        && advance.advanced
        && !advance.reached_range_start;
    Ok(TakeoutPage::from_parts(
        profile,
        advance.cursor,
        page_index.saturating_add(1),
        messages.into_iter().map(TakeoutMessage::from_raw).collect(),
        has_next,
        only_my_messages,
        None,
    ))
}

#[cfg(test)]
mod tests {
    use super::super::types::{MessageRange, TakeoutMessage};
    use super::{
        parse_takeout_page, select_history_splits, takeout_page_request, TakeoutPaginationCursor,
        TakeoutPaginationProfile, TAKEOUT_HISTORY_PAGE_LIMIT,
    };
    use extractum_core::error::AppErrorKind;
    use grammers_client::tl;

    #[test]
    fn split_selection_uses_last_range_for_channel_and_supergroup() {
        let ranges = vec![raw_range(1, 10), raw_range(11, 20)];
        let channel = select_history_splits("channel", ranges.clone()).expect("channel splits");
        let supergroup = select_history_splits("supergroup", ranges).expect("supergroup splits");
        assert_eq!(channel, vec![MessageRange::new(11, 20)]);
        assert_eq!(supergroup, vec![MessageRange::new(11, 20)]);
    }

    #[test]
    fn split_selection_uses_all_ranges_for_small_group() {
        let selected = select_history_splits("group", vec![raw_range(1, 10), raw_range(11, 20)])
            .expect("group splits");
        assert_eq!(
            selected,
            vec![MessageRange::new(1, 10), MessageRange::new(11, 20)]
        );
    }

    #[test]
    fn split_selection_falls_back_when_telegram_returns_no_ranges() {
        let selected = select_history_splits("group", Vec::new()).expect("fallback split");
        assert_eq!(selected, vec![MessageRange::new(1, i32::MAX)]);
    }

    #[test]
    fn tdesktop_pagination_reverses_raw_order_and_advances_from_newest_id() {
        let range = MessageRange::new(1, 1_000);
        let cursor = TakeoutPaginationCursor::new(TakeoutPaginationProfile::TDesktop, &range);
        let request = takeout_page_request(cursor);
        assert_eq!(
            (request.offset_id, request.add_offset, request.limit),
            (1, -TAKEOUT_HISTORY_PAGE_LIMIT, TAKEOUT_HISTORY_PAGE_LIMIT)
        );

        let mut page = parse_takeout_page(
            messages_slice_response(vec![300, 250, 200]),
            TakeoutPaginationProfile::TDesktop,
            cursor,
            &range,
            3,
            0,
            false,
        )
        .expect("parse tdesktop page");
        assert_eq!(message_ids(&page.take_messages()), vec![200, 250, 300]);
        let (_, next_cursor, next_page_index) = page.pagination_state();
        assert_eq!(
            next_cursor,
            TakeoutPaginationCursor::TDesktop {
                largest_id_plus_one: 301
            }
        );
        assert_eq!(next_page_index, 1);
        assert!(page.has_next());
        let next_request = takeout_page_request(next_cursor);
        assert_eq!(next_request.offset_id, 301);
    }

    #[test]
    fn descending_fallback_keeps_raw_order_and_moves_to_min_message_id() {
        let range = MessageRange::new(1, 1_000);
        let cursor =
            TakeoutPaginationCursor::new(TakeoutPaginationProfile::DescendingFallback, &range);
        let mut page = parse_takeout_page(
            messages_slice_response(vec![999, 900, 850]),
            TakeoutPaginationProfile::DescendingFallback,
            cursor,
            &range,
            3,
            0,
            false,
        )
        .expect("parse descending page");
        assert_eq!(message_ids(&page.take_messages()), vec![999, 900, 850]);
        let (_, next_cursor, _) = page.pagination_state();
        assert_eq!(
            next_cursor,
            TakeoutPaginationCursor::DescendingFallback { offset_id: 850 }
        );
        assert!(page.has_next());
        let next_request = takeout_page_request(next_cursor);
        assert_eq!((next_request.offset_id, next_request.add_offset), (850, 0));
    }

    #[test]
    fn tdesktop_empty_first_page_with_nonzero_count_restarts_descending_fallback() {
        let range = MessageRange::new(10, 500);
        let cursor = TakeoutPaginationCursor::new(TakeoutPaginationProfile::TDesktop, &range);
        let mut page = parse_takeout_page(
            messages_slice_response(Vec::new()),
            TakeoutPaginationProfile::TDesktop,
            cursor,
            &range,
            25,
            0,
            false,
        )
        .expect("parse empty page");
        assert!(page.take_messages().is_empty());
        assert!(page.has_next());
        let warning = page
            .take_pagination_fallback_warning()
            .expect("pagination fallback warning");
        assert!(warning.contains("TDesktop Takeout pagination"));
        assert!(warning.contains("10..500"));
        assert!(warning.contains("descending fallback"));
        assert_eq!(
            page.pagination_state(),
            (
                TakeoutPaginationProfile::DescendingFallback,
                TakeoutPaginationCursor::DescendingFallback { offset_id: 500 },
                0,
            )
        );

        let no_restart = parse_takeout_page(
            messages_slice_response(Vec::new()),
            TakeoutPaginationProfile::TDesktop,
            cursor,
            &range,
            0,
            0,
            false,
        )
        .expect("parse empty zero-count page");
        assert!(!no_restart.has_next());
    }

    #[test]
    fn tdesktop_non_advancing_cursor_restarts_descending_fallback() {
        let range = MessageRange::new(1, 1_000);
        let cursor = TakeoutPaginationCursor::TDesktop {
            largest_id_plus_one: 301,
        };
        let mut page = parse_takeout_page(
            messages_slice_response(vec![300, 200, 100]),
            TakeoutPaginationProfile::TDesktop,
            cursor,
            &range,
            25,
            3,
            false,
        )
        .expect("parse page");
        assert!(page.take_messages().is_empty());
        assert!(page.has_next());
        assert!(page.take_pagination_fallback_warning().is_some());
        assert_eq!(
            page.pagination_state().1,
            TakeoutPaginationCursor::DescendingFallback { offset_id: 1_000 }
        );
    }

    #[test]
    fn messages_response_without_slice_is_terminal_page() {
        let range = MessageRange::new(1, 1_000);
        let cursor = TakeoutPaginationCursor::new(TakeoutPaginationProfile::TDesktop, &range);
        let mut page = parse_takeout_page(
            messages_messages_response(vec![30, 20, 10]),
            TakeoutPaginationProfile::TDesktop,
            cursor,
            &range,
            3,
            0,
            false,
        )
        .expect("parse terminal page");
        assert!(!page.has_next());
        assert_eq!(message_ids(&page.take_messages()), vec![10, 20, 30]);
    }

    #[test]
    fn messages_not_modified_response_is_rejected_for_takeout_page() {
        let range = MessageRange::new(1, 1_000);
        let cursor = TakeoutPaginationCursor::new(TakeoutPaginationProfile::TDesktop, &range);
        let error = parse_takeout_page(
            tl::enums::messages::Messages::NotModified(tl::types::messages::MessagesNotModified {
                count: 0,
            }),
            TakeoutPaginationProfile::TDesktop,
            cursor,
            &range,
            0,
            0,
            false,
        )
        .expect_err("messagesNotModified should fail");
        assert_eq!(error.kind, AppErrorKind::Network);
        assert_eq!(
            error.message,
            "Telegram returned messagesNotModified for Takeout history page"
        );
    }

    fn raw_range(min_id: i32, max_id: i32) -> tl::enums::MessageRange {
        tl::types::MessageRange { min_id, max_id }.into()
    }

    fn message_ids(messages: &[TakeoutMessage]) -> Vec<i32> {
        messages
            .iter()
            .map(|message| message.message_id() as i32)
            .collect()
    }

    fn messages_slice_response(ids: Vec<i32>) -> tl::enums::messages::Messages {
        tl::types::messages::MessagesSlice {
            inexact: false,
            count: ids.len() as i32,
            next_rate: None,
            offset_id_offset: None,
            search_flood: None,
            messages: ids
                .into_iter()
                .map(raw_message)
                .map(tl::enums::Message::Message)
                .collect(),
            topics: Vec::new(),
            chats: Vec::new(),
            users: Vec::new(),
        }
        .into()
    }

    fn messages_messages_response(ids: Vec<i32>) -> tl::enums::messages::Messages {
        tl::types::messages::Messages {
            messages: ids
                .into_iter()
                .map(raw_message)
                .map(tl::enums::Message::Message)
                .collect(),
            topics: Vec::new(),
            chats: Vec::new(),
            users: Vec::new(),
        }
        .into()
    }

    fn raw_message(id: i32) -> tl::types::Message {
        tl::types::Message {
            out: false,
            mentioned: false,
            media_unread: false,
            silent: false,
            post: false,
            from_scheduled: false,
            legacy: false,
            edit_hide: false,
            pinned: false,
            noforwards: false,
            invert_media: false,
            offline: false,
            video_processing_pending: false,
            paid_suggested_post_stars: false,
            paid_suggested_post_ton: false,
            id,
            from_id: None,
            from_boosts_applied: None,
            from_rank: None,
            rich_message: None,
            peer_id: tl::types::PeerChannel { channel_id: 10 }.into(),
            saved_peer_id: None,
            fwd_from: None,
            via_bot_id: None,
            via_business_bot_id: None,
            guestchat_via_from: None,
            reply_to: None,
            date: 1234,
            message: String::new(),
            media: None,
            reply_markup: None,
            entities: None,
            views: None,
            forwards: None,
            replies: None,
            edit_date: None,
            post_author: None,
            grouped_id: None,
            reactions: None,
            restriction_reason: None,
            ttl_period: None,
            quick_reply_shortcut_id: None,
            effect: None,
            factcheck: None,
            report_delivery_until_date: None,
            paid_message_stars: None,
            suggested_post: None,
            schedule_repeat_period: None,
            summary_from_language: None,
        }
    }
}
