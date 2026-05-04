use grammers_client::tl;

use crate::error::{AppError, AppResult};
use crate::sources::{TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP, TELEGRAM_KIND_SUPERGROUP};

const TAKEOUT_HISTORY_PAGE_LIMIT: i32 = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TakeoutPaginationProfile {
    TDesktop,
    DescendingFallback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TakeoutPageRequest {
    pub(crate) offset_id: i32,
    pub(crate) add_offset: i32,
    pub(crate) limit: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TakeoutPaginationCursor {
    TDesktop { largest_id_plus_one: i32 },
    DescendingFallback { offset_id: i32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TakeoutCursorAdvance {
    pub(crate) cursor: TakeoutPaginationCursor,
    pub(crate) advanced: bool,
    pub(crate) reached_range_start: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ParsedTakeoutPage {
    pub(crate) messages: Vec<tl::types::Message>,
    pub(crate) first_regular_message_id: Option<i32>,
    pub(crate) last_regular_message_id: Option<i32>,
    pub(crate) oldest_regular_message_id: Option<i32>,
    pub(crate) newest_regular_message_id: Option<i32>,
    pub(crate) is_terminal_response: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TakeoutPaginationFallbackReason {
    EmptyFirstPageWithNonZeroCount,
    NonAdvancingTDesktopCursor,
}

pub(crate) fn select_history_splits(
    telegram_source_kind: &str,
    split_ranges: Vec<tl::enums::MessageRange>,
) -> AppResult<Vec<tl::enums::MessageRange>> {
    let mut ranges = if split_ranges.is_empty() {
        vec![fallback_message_range()]
    } else {
        split_ranges
    };

    match telegram_source_kind {
        TELEGRAM_KIND_CHANNEL | TELEGRAM_KIND_SUPERGROUP => {
            Ok(vec![ranges.pop().unwrap_or_else(fallback_message_range)])
        }
        TELEGRAM_KIND_GROUP => Ok(ranges),
        other => Err(AppError::validation(format!(
            "Unsupported telegram_source_kind '{other}'"
        ))),
    }
}

fn fallback_message_range() -> tl::enums::MessageRange {
    tl::types::MessageRange {
        min_id: 1,
        max_id: i32::MAX,
    }
    .into()
}

// This deliberately models the full TDesktop history pagination cycle instead of
// only swapping Extractum's old `add_offset = 0` for `add_offset = -100`.
// TDesktop starts every split with `largestIdPlusOne = 1`, requests the page
// above that cursor with `add_offset = -limit`, then `ParseMessagesSlice`
// reverses Telegram's raw newest-to-oldest response into oldest-to-newest order.
// Only after that reversal does `finishMessagesSlice` move the cursor to the
// newest message from the parsed page plus one.
//
// The `DescendingFallback` profile is Extractum-specific. A live Takeout run
// showed that the naive TDesktop-looking request (`offset_id=1/add_offset=-100`)
// can return an empty first page through the current grammers raw Takeout request
// shape, while the descending `offset_id=range.max_id/add_offset=0` profile did
// import the real channel history. Keeping both profiles explicit makes the
// TDesktop path auditable and protects the known-working recovery path from a
// future "simplification" that would break real imports again.
impl TakeoutPaginationCursor {
    pub(crate) fn new(profile: TakeoutPaginationProfile, range: &tl::enums::MessageRange) -> Self {
        match profile {
            TakeoutPaginationProfile::TDesktop => Self::TDesktop {
                largest_id_plus_one: 1,
            },
            TakeoutPaginationProfile::DescendingFallback => Self::DescendingFallback {
                offset_id: message_range_max_id(range),
            },
        }
    }
}

pub(crate) fn takeout_page_request(cursor: TakeoutPaginationCursor) -> TakeoutPageRequest {
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

pub(crate) fn next_takeout_cursor(
    cursor: TakeoutPaginationCursor,
    page: &ParsedTakeoutPage,
    range: &tl::enums::MessageRange,
) -> TakeoutCursorAdvance {
    let min_id = message_range_min_id(range);
    match cursor {
        TakeoutPaginationCursor::TDesktop {
            largest_id_plus_one,
        } => {
            let next_largest_id_plus_one = page
                .messages
                .iter()
                .map(|message| message.id)
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
            let next_offset_id = page
                .messages
                .iter()
                .map(|message| message.id)
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

pub(crate) fn should_restart_with_descending_fallback(
    profile: TakeoutPaginationProfile,
    split_count: i64,
    page_index: usize,
    page: &ParsedTakeoutPage,
    advance: TakeoutCursorAdvance,
) -> Option<TakeoutPaginationFallbackReason> {
    if profile != TakeoutPaginationProfile::TDesktop {
        return None;
    }

    if page_index == 0 && split_count > 0 && page.messages.is_empty() {
        return Some(TakeoutPaginationFallbackReason::EmptyFirstPageWithNonZeroCount);
    }

    if !page.messages.is_empty() && !advance.advanced {
        return Some(TakeoutPaginationFallbackReason::NonAdvancingTDesktopCursor);
    }

    None
}

pub(crate) fn takeout_pagination_fallback_warning(
    reason: TakeoutPaginationFallbackReason,
    range: &tl::enums::MessageRange,
) -> String {
    let reason = match reason {
        TakeoutPaginationFallbackReason::EmptyFirstPageWithNonZeroCount => "an empty first page",
        TakeoutPaginationFallbackReason::NonAdvancingTDesktopCursor => "a non-advancing cursor",
    };
    format!(
        "TDesktop Takeout pagination returned {reason} for split {}..{}; retrying this split with Extractum descending fallback.",
        message_range_min_id(range),
        message_range_max_id(range)
    )
}

pub(crate) fn parse_takeout_page(
    response: tl::enums::messages::Messages,
    profile: TakeoutPaginationProfile,
) -> AppResult<ParsedTakeoutPage> {
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
        // TDesktop's cursor update depends on this order: after reversing,
        // `messages.last()` is the newest page item, so `newest + 1` matches
        // `slice.list.back().id + 1` from `finishMessagesSlice`.
        messages.reverse();
    }

    let first_regular_message_id = messages.first().map(|message| message.id);
    let last_regular_message_id = messages.last().map(|message| message.id);
    let oldest_regular_message_id = messages.iter().map(|message| message.id).min();
    let newest_regular_message_id = messages.iter().map(|message| message.id).max();

    Ok(ParsedTakeoutPage {
        messages,
        first_regular_message_id,
        last_regular_message_id,
        oldest_regular_message_id,
        newest_regular_message_id,
        is_terminal_response,
    })
}

pub(crate) fn message_range_min_id(range: &tl::enums::MessageRange) -> i32 {
    match range {
        tl::enums::MessageRange::Range(range) => range.min_id,
    }
}

pub(crate) fn message_range_max_id(range: &tl::enums::MessageRange) -> i32 {
    match range {
        tl::enums::MessageRange::Range(range) => range.max_id,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        message_range_max_id, message_range_min_id, next_takeout_cursor, parse_takeout_page,
        select_history_splits, should_restart_with_descending_fallback, takeout_page_request,
        takeout_pagination_fallback_warning, TakeoutPaginationCursor,
        TakeoutPaginationFallbackReason, TakeoutPaginationProfile, TAKEOUT_HISTORY_PAGE_LIMIT,
    };
    use crate::sources::{TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP, TELEGRAM_KIND_SUPERGROUP};
    use grammers_client::tl;

    #[test]
    fn split_selection_uses_last_range_for_channel_and_supergroup() {
        let ranges = vec![message_range(1, 10), message_range(11, 20)];

        let channel =
            select_history_splits(TELEGRAM_KIND_CHANNEL, ranges.clone()).expect("channel splits");
        let supergroup =
            select_history_splits(TELEGRAM_KIND_SUPERGROUP, ranges).expect("supergroup splits");

        assert_eq!(channel.len(), 1);
        assert_eq!(message_range_min_id(&channel[0]), 11);
        assert_eq!(message_range_max_id(&channel[0]), 20);
        assert_eq!(supergroup.len(), 1);
        assert_eq!(message_range_min_id(&supergroup[0]), 11);
        assert_eq!(message_range_max_id(&supergroup[0]), 20);
    }

    #[test]
    fn split_selection_uses_all_ranges_for_small_group() {
        let ranges = vec![message_range(1, 10), message_range(11, 20)];

        let selected = select_history_splits(TELEGRAM_KIND_GROUP, ranges).expect("group splits");

        assert_eq!(selected.len(), 2);
        assert_eq!(message_range_min_id(&selected[0]), 1);
        assert_eq!(message_range_max_id(&selected[0]), 10);
        assert_eq!(message_range_min_id(&selected[1]), 11);
        assert_eq!(message_range_max_id(&selected[1]), 20);
    }

    #[test]
    fn split_selection_falls_back_when_telegram_returns_no_ranges() {
        let selected =
            select_history_splits(TELEGRAM_KIND_GROUP, Vec::new()).expect("fallback split");

        assert_eq!(selected.len(), 1);
        assert_eq!(message_range_min_id(&selected[0]), 1);
        assert_eq!(message_range_max_id(&selected[0]), i32::MAX);
    }

    #[test]
    fn tdesktop_pagination_reverses_raw_order_and_advances_from_newest_id() {
        let range = message_range(1, 1_000);
        let cursor = TakeoutPaginationCursor::new(TakeoutPaginationProfile::TDesktop, &range);
        let request = takeout_page_request(cursor);

        assert_eq!(request.offset_id, 1);
        assert_eq!(request.add_offset, -TAKEOUT_HISTORY_PAGE_LIMIT);
        assert_eq!(request.limit, TAKEOUT_HISTORY_PAGE_LIMIT);

        let page = parse_takeout_page(
            messages_slice_response(vec![300, 250, 200]),
            TakeoutPaginationProfile::TDesktop,
        )
        .expect("parse tdesktop page");

        assert_eq!(message_ids(&page.messages), vec![200, 250, 300]);
        assert_eq!(page.oldest_regular_message_id, Some(200));
        assert_eq!(page.newest_regular_message_id, Some(300));

        let advance = next_takeout_cursor(cursor, &page, &range);
        assert!(advance.advanced);
        assert_eq!(
            advance.cursor,
            TakeoutPaginationCursor::TDesktop {
                largest_id_plus_one: 301
            }
        );

        let next_request = takeout_page_request(advance.cursor);
        assert_eq!(next_request.offset_id, 301);
        assert_eq!(next_request.add_offset, -TAKEOUT_HISTORY_PAGE_LIMIT);
        assert_eq!(next_request.limit, TAKEOUT_HISTORY_PAGE_LIMIT);
    }

    #[test]
    fn descending_fallback_keeps_raw_order_and_moves_to_min_message_id() {
        let range = message_range(1, 1_000);
        let cursor =
            TakeoutPaginationCursor::new(TakeoutPaginationProfile::DescendingFallback, &range);
        let request = takeout_page_request(cursor);

        assert_eq!(request.offset_id, 1_000);
        assert_eq!(request.add_offset, 0);
        assert_eq!(request.limit, TAKEOUT_HISTORY_PAGE_LIMIT);

        let page = parse_takeout_page(
            messages_slice_response(vec![999, 900, 850]),
            TakeoutPaginationProfile::DescendingFallback,
        )
        .expect("parse descending page");

        assert_eq!(message_ids(&page.messages), vec![999, 900, 850]);

        let advance = next_takeout_cursor(cursor, &page, &range);
        assert!(advance.advanced);
        assert!(!advance.reached_range_start);
        assert_eq!(
            advance.cursor,
            TakeoutPaginationCursor::DescendingFallback { offset_id: 850 }
        );

        let next_request = takeout_page_request(advance.cursor);
        assert_eq!(next_request.offset_id, 850);
        assert_eq!(next_request.add_offset, 0);
        assert_eq!(next_request.limit, TAKEOUT_HISTORY_PAGE_LIMIT);
    }

    #[test]
    fn tdesktop_empty_first_page_with_nonzero_count_restarts_descending_fallback() {
        let range = message_range(10, 500);
        let cursor = TakeoutPaginationCursor::new(TakeoutPaginationProfile::TDesktop, &range);
        let page = parse_takeout_page(
            messages_slice_response(Vec::new()),
            TakeoutPaginationProfile::TDesktop,
        )
        .expect("parse empty page");
        let advance = next_takeout_cursor(cursor, &page, &range);

        assert_eq!(
            should_restart_with_descending_fallback(
                TakeoutPaginationProfile::TDesktop,
                25,
                0,
                &page,
                advance,
            ),
            Some(TakeoutPaginationFallbackReason::EmptyFirstPageWithNonZeroCount)
        );
        assert_eq!(
            should_restart_with_descending_fallback(
                TakeoutPaginationProfile::TDesktop,
                0,
                0,
                &page,
                advance,
            ),
            None
        );

        let warning = takeout_pagination_fallback_warning(
            TakeoutPaginationFallbackReason::EmptyFirstPageWithNonZeroCount,
            &range,
        );
        assert!(warning.contains("TDesktop Takeout pagination"));
        assert!(warning.contains("10..500"));
        assert!(warning.contains("descending fallback"));
    }

    #[test]
    fn tdesktop_non_advancing_cursor_restarts_descending_fallback() {
        let range = message_range(1, 1_000);
        let cursor = TakeoutPaginationCursor::TDesktop {
            largest_id_plus_one: 301,
        };
        let page = parse_takeout_page(
            messages_slice_response(vec![300, 200, 100]),
            TakeoutPaginationProfile::TDesktop,
        )
        .expect("parse page");
        let advance = next_takeout_cursor(cursor, &page, &range);

        assert_eq!(message_ids(&page.messages), vec![100, 200, 300]);
        assert!(!advance.advanced);
        assert_eq!(
            should_restart_with_descending_fallback(
                TakeoutPaginationProfile::TDesktop,
                25,
                3,
                &page,
                advance,
            ),
            Some(TakeoutPaginationFallbackReason::NonAdvancingTDesktopCursor)
        );
    }

    #[test]
    fn messages_response_without_slice_is_terminal_page() {
        let page = parse_takeout_page(
            messages_messages_response(vec![30, 20, 10]),
            TakeoutPaginationProfile::TDesktop,
        )
        .expect("parse terminal page");

        assert!(page.is_terminal_response);
        assert_eq!(message_ids(&page.messages), vec![10, 20, 30]);
    }

    #[test]
    fn messages_not_modified_response_is_rejected_for_takeout_page() {
        let error = parse_takeout_page(
            tl::enums::messages::Messages::NotModified(tl::types::messages::MessagesNotModified {
                count: 0,
            }),
            TakeoutPaginationProfile::TDesktop,
        )
        .expect_err("messagesNotModified should fail");

        assert!(error
            .message
            .contains("Telegram returned messagesNotModified for Takeout history page"));
    }

    fn message_range(min_id: i32, max_id: i32) -> tl::enums::MessageRange {
        tl::types::MessageRange { min_id, max_id }.into()
    }

    fn message_ids(messages: &[tl::types::Message]) -> Vec<i32> {
        messages.iter().map(|message| message.id).collect()
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
            peer_id: tl::types::PeerChannel { channel_id: 10 }.into(),
            saved_peer_id: None,
            fwd_from: None,
            via_bot_id: None,
            via_business_bot_id: None,
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
