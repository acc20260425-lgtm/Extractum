use grammers_client::tl;
use serde_json::json;

use crate::media::{
    derive_content_kind, derive_document_media_kind, media_label, DocumentSignals,
    ExtractedItemPayload, ExtractedMediaPayload, ItemMediaMetadata,
};
use crate::sources::{
    SourceItemInsert, TelegramItemContext, TelegramMessageIdentity, ITEM_KIND_TELEGRAM_MESSAGE,
};

pub(crate) fn parse_raw_message(
    source_title: &Option<String>,
    message: tl::types::Message,
) -> Result<Option<SourceItemInsert>, String> {
    let content = trimmed_non_empty(&message.message);
    let media = message.media.as_ref().and_then(extract_raw_media_payload);
    let has_content = content.is_some();
    let has_media = media.is_some();

    if !has_content && !has_media {
        return Ok(None);
    }

    let content_kind = derive_content_kind(has_content, has_media);
    let payload = ExtractedItemPayload {
        content,
        content_kind,
        media,
    };
    let author = raw_message_author(&message);
    let telegram_context = extract_raw_telegram_context(&message);
    let telegram_identity = raw_message_identity(&message);
    let raw_data = serde_json::to_vec(&json!({
        "id": message.id,
        "peer_id": peer_id_string(&message.peer_id),
        "sender_id": message.from_id.as_ref().map(peer_id_string),
        "published_at": message.date,
        "text": payload.content.as_deref(),
        "content_kind": payload.content_kind,
        "has_media": payload.media.is_some(),
        "media_kind": payload.media.as_ref().map(|media| &media.kind),
        "media_metadata": payload.media.as_ref().map(|media| &media.metadata),
        "post_author": message.post_author.as_deref(),
        "source_title": source_title.as_deref(),
        "author": author.as_deref(),
    }))
    .map_err(|error| error.to_string())?;

    Ok(Some(SourceItemInsert {
        external_id: message.id.to_string(),
        item_kind: ITEM_KIND_TELEGRAM_MESSAGE.to_string(),
        author,
        published_at: i64::from(message.date),
        payload,
        raw_data,
        telegram_context,
        telegram_identity: Some(telegram_identity),
    }))
}

fn extract_raw_media_payload(media: &tl::enums::MessageMedia) -> Option<ExtractedMediaPayload> {
    match media {
        tl::enums::MessageMedia::Photo(photo) => extract_photo_media_payload(photo),
        tl::enums::MessageMedia::Document(document) => extract_document_media_payload(document),
        tl::enums::MessageMedia::Contact(contact) => Some(ExtractedMediaPayload {
            kind: "contact".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some(contact_summary(contact)),
                ..ItemMediaMetadata::default()
            },
        }),
        tl::enums::MessageMedia::Poll(_) => Some(ExtractedMediaPayload {
            kind: "poll".to_string(),
            metadata: ItemMediaMetadata {
                summary: Some("Poll".to_string()),
                ..ItemMediaMetadata::default()
            },
        }),
        tl::enums::MessageMedia::Geo(_) => Some(raw_summary_media("location", "Location")),
        tl::enums::MessageMedia::GeoLive(_) => {
            Some(raw_summary_media("live_location", "Live location"))
        }
        tl::enums::MessageMedia::Venue(venue) => Some(ExtractedMediaPayload {
            kind: "venue".to_string(),
            metadata: ItemMediaMetadata {
                summary: trimmed_non_empty(&venue.title).or_else(|| Some("Venue".to_string())),
                ..ItemMediaMetadata::default()
            },
        }),
        tl::enums::MessageMedia::WebPage(_) => {
            Some(raw_summary_media("webpage", "Web page preview"))
        }
        tl::enums::MessageMedia::Dice(_) => Some(raw_summary_media("dice", "Dice")),
        tl::enums::MessageMedia::Empty => None,
        _ => Some(raw_summary_media("document", "Media")),
    }
}

fn extract_photo_media_payload(
    media: &tl::types::MessageMediaPhoto,
) -> Option<ExtractedMediaPayload> {
    let photo = match media.photo.as_ref()? {
        tl::enums::Photo::Photo(photo) => photo,
        tl::enums::Photo::Empty(_) => {
            return Some(raw_summary_media("photo", "Photo"));
        }
    };
    let mut metadata = ItemMediaMetadata {
        summary: Some("Photo".to_string()),
        ..ItemMediaMetadata::default()
    };

    for size in &photo.sizes {
        match size {
            tl::enums::PhotoSize::Size(size) => {
                apply_larger_photo_size(&mut metadata, size.w, size.h, Some(i64::from(size.size)))
            }
            tl::enums::PhotoSize::PhotoCachedSize(size) => apply_larger_photo_size(
                &mut metadata,
                size.w,
                size.h,
                i64::try_from(size.bytes.len()).ok(),
            ),
            tl::enums::PhotoSize::Progressive(size) => apply_larger_photo_size(
                &mut metadata,
                size.w,
                size.h,
                size.sizes.iter().copied().max().map(i64::from),
            ),
            _ => {}
        }
    }

    Some(ExtractedMediaPayload {
        kind: "photo".to_string(),
        metadata,
    })
}

fn extract_document_media_payload(
    media: &tl::types::MessageMediaDocument,
) -> Option<ExtractedMediaPayload> {
    let document = match media.document.as_ref()? {
        tl::enums::Document::Document(document) => document,
        tl::enums::Document::Empty(_) => return Some(raw_summary_media("document", "Document")),
    };
    let mut signals = DocumentSignals {
        mime_type: Some(document.mime_type.clone()),
        ..DocumentSignals::default()
    };
    let mut file_name = None;
    let mut width = None;
    let mut height = None;
    let mut duration_seconds = None;
    let mut sticker_alt = None;

    for attribute in &document.attributes {
        match attribute {
            tl::enums::DocumentAttribute::Animated => signals.is_animated = true,
            tl::enums::DocumentAttribute::Audio(audio) => {
                signals.has_audio = true;
                signals.is_voice = audio.voice;
                duration_seconds = Some(f64::from(audio.duration));
            }
            tl::enums::DocumentAttribute::Filename(name) => {
                file_name = trimmed_non_empty(&name.file_name);
            }
            tl::enums::DocumentAttribute::ImageSize(size) => {
                width = Some(size.w);
                height = Some(size.h);
            }
            tl::enums::DocumentAttribute::Sticker(sticker) => {
                sticker_alt = trimmed_non_empty(&sticker.alt);
            }
            tl::enums::DocumentAttribute::Video(video) => {
                signals.has_video = true;
                width = Some(video.w);
                height = Some(video.h);
                duration_seconds = Some(video.duration);
            }
            _ => {}
        }
    }

    if media.video {
        signals.has_video = true;
    }
    if media.voice {
        signals.has_audio = true;
        signals.is_voice = true;
    }

    let mut kind = derive_document_media_kind(&signals).to_string();
    let mut summary = media_label(&kind).to_string();
    if let Some(alt) = sticker_alt {
        kind = "sticker".to_string();
        summary = format!("Sticker {alt}");
    }

    Some(ExtractedMediaPayload {
        kind,
        metadata: ItemMediaMetadata {
            summary: Some(summary),
            file_name,
            mime_type: Some(document.mime_type.clone()),
            size_bytes: Some(document.size),
            width,
            height,
            duration_seconds,
        },
    })
}

fn extract_raw_telegram_context(message: &tl::types::Message) -> TelegramItemContext {
    let mut context = TelegramItemContext {
        reaction_count: message.reactions.as_ref().map(reaction_count),
        ..TelegramItemContext::default()
    };

    if let Some(tl::enums::MessageReplyHeader::Header(header)) = message.reply_to.as_ref() {
        context.reply_to_msg_id = header.reply_to_msg_id.map(i64::from);
        context.reply_to_top_id = header.reply_to_top_id.map(i64::from);
        if let Some((kind, id)) = peer_context(header.reply_to_peer_id.as_ref()) {
            context.reply_to_peer_kind = Some(kind.to_string());
            context.reply_to_peer_id = Some(id);
        }
    }

    context
}

fn raw_message_identity(message: &tl::types::Message) -> TelegramMessageIdentity {
    let (history_peer_kind, history_peer_id) = match &message.peer_id {
        tl::enums::Peer::User(peer) => ("user", peer.user_id),
        tl::enums::Peer::Chat(peer) => ("chat", peer.chat_id),
        tl::enums::Peer::Channel(peer) => ("channel", peer.channel_id),
    };

    TelegramMessageIdentity {
        history_peer_kind: history_peer_kind.to_string(),
        history_peer_id,
        telegram_message_id: i64::from(message.id),
        migration_domain: None,
        is_migrated_history: false,
    }
}

fn reaction_count(reactions: &tl::enums::MessageReactions) -> i64 {
    match reactions {
        tl::enums::MessageReactions::Reactions(reactions) => reactions
            .results
            .iter()
            .map(|count| match count {
                tl::enums::ReactionCount::Count(count) => i64::from(count.count),
            })
            .sum(),
    }
}

fn raw_message_author(message: &tl::types::Message) -> Option<String> {
    message
        .post_author
        .as_ref()
        .and_then(|author| trimmed_non_empty(author))
        .or_else(|| message.from_id.as_ref().map(peer_id_string))
}

fn peer_context(peer: Option<&tl::enums::Peer>) -> Option<(&'static str, String)> {
    match peer? {
        tl::enums::Peer::User(peer) => Some(("user", peer.user_id.to_string())),
        tl::enums::Peer::Chat(peer) => Some(("chat", peer.chat_id.to_string())),
        tl::enums::Peer::Channel(peer) => Some(("channel", peer.channel_id.to_string())),
    }
}

fn peer_id_string(peer: &tl::enums::Peer) -> String {
    match peer {
        tl::enums::Peer::User(peer) => format!("user:{}", peer.user_id),
        tl::enums::Peer::Chat(peer) => format!("chat:{}", peer.chat_id),
        tl::enums::Peer::Channel(peer) => format!("channel:{}", peer.channel_id),
    }
}

fn raw_summary_media(kind: &str, summary: &str) -> ExtractedMediaPayload {
    ExtractedMediaPayload {
        kind: kind.to_string(),
        metadata: ItemMediaMetadata {
            summary: Some(summary.to_string()),
            ..ItemMediaMetadata::default()
        },
    }
}

fn apply_larger_photo_size(
    metadata: &mut ItemMediaMetadata,
    width: i32,
    height: i32,
    size_bytes: Option<i64>,
) {
    let current_area = metadata
        .width
        .zip(metadata.height)
        .map(|(width, height)| i64::from(width) * i64::from(height))
        .unwrap_or(0);
    let area = i64::from(width) * i64::from(height);

    if area >= current_area {
        metadata.width = Some(width);
        metadata.height = Some(height);
        metadata.size_bytes = size_bytes;
    }
}

fn contact_summary(contact: &tl::types::MessageMediaContact) -> String {
    let display_name = [&contact.first_name, &contact.last_name]
        .into_iter()
        .filter_map(|part| trimmed_non_empty(part))
        .collect::<Vec<_>>()
        .join(" ");

    if !display_name.is_empty() {
        return format!("Contact: {display_name}");
    }

    trimmed_non_empty(&contact.phone_number)
        .map(|phone| format!("Contact: {phone}"))
        .unwrap_or_else(|| "Contact card".to_string())
}

fn trimmed_non_empty(input: &str) -> Option<String> {
    let trimmed = input.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::parse_raw_message;
    use grammers_client::tl;

    fn peer_channel(channel_id: i64) -> tl::enums::Peer {
        tl::types::PeerChannel { channel_id }.into()
    }

    fn peer_user(user_id: i64) -> tl::enums::Peer {
        tl::types::PeerUser { user_id }.into()
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
            peer_id: peer_channel(10),
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

    #[test]
    fn parses_text_message_with_reply_and_reactions() {
        let mut message = raw_message(42);
        message.message = " hello ".to_string();
        message.from_id = Some(peer_user(77));
        message.reply_to = Some(
            tl::types::MessageReplyHeader {
                reply_to_scheduled: false,
                forum_topic: false,
                quote: false,
                reply_to_msg_id: Some(7),
                reply_to_peer_id: Some(peer_channel(99)),
                reply_from: None,
                reply_media: None,
                reply_to_top_id: Some(5),
                quote_text: None,
                quote_entities: None,
                quote_offset: None,
                todo_item_id: None,
                poll_option: None,
            }
            .into(),
        );
        message.reactions = Some(
            tl::types::MessageReactions {
                min: false,
                can_see_list: false,
                reactions_as_tags: false,
                results: vec![tl::types::ReactionCount {
                    chosen_order: None,
                    reaction: tl::types::ReactionEmoji {
                        emoticon: "👍".to_string(),
                    }
                    .into(),
                    count: 3,
                }
                .into()],
                recent_reactions: None,
                top_reactors: None,
            }
            .into(),
        );

        let item = parse_raw_message(&Some("Channel".to_string()), message)
            .expect("parse raw message")
            .expect("message item");

        assert_eq!(item.external_id, "42");
        assert_eq!(item.author.as_deref(), Some("user:77"));
        assert_eq!(item.published_at, 1234);
        assert_eq!(item.payload.content.as_deref(), Some("hello"));
        assert_eq!(item.payload.content_kind, "text_only");
        assert!(item.payload.media.is_none());
        assert_eq!(item.telegram_context.reply_to_msg_id, Some(7));
        assert_eq!(
            item.telegram_context.reply_to_peer_kind.as_deref(),
            Some("channel")
        );
        assert_eq!(
            item.telegram_context.reply_to_peer_id.as_deref(),
            Some("99")
        );
        assert_eq!(item.telegram_context.reply_to_top_id, Some(5));
        assert_eq!(item.telegram_context.reaction_count, Some(3));
        assert_eq!(
            item.telegram_identity
                .as_ref()
                .expect("identity")
                .history_peer_kind,
            "channel"
        );
        assert_eq!(
            item.telegram_identity
                .as_ref()
                .expect("identity")
                .history_peer_id,
            10
        );
        assert_eq!(
            item.telegram_identity
                .as_ref()
                .expect("identity")
                .telegram_message_id,
            42
        );
    }

    #[test]
    fn parse_raw_message_carries_raw_history_peer_for_overlapping_message_ids() {
        let mut current = raw_message(42);
        current.message = "current".to_string();
        current.peer_id = peer_channel(12345);
        let mut migrated = raw_message(42);
        migrated.message = "migrated".to_string();
        migrated.peer_id = tl::types::PeerChat { chat_id: 777 }.into();

        let current_item = parse_raw_message(&None, current)
            .expect("parse current")
            .expect("current item");
        let migrated_item = parse_raw_message(&None, migrated)
            .expect("parse migrated")
            .expect("migrated item");

        assert_eq!(
            current_item
                .telegram_identity
                .as_ref()
                .unwrap()
                .history_peer_kind,
            "channel"
        );
        assert_eq!(
            migrated_item
                .telegram_identity
                .as_ref()
                .unwrap()
                .history_peer_kind,
            "chat"
        );
        assert_eq!(
            migrated_item
                .telegram_identity
                .as_ref()
                .unwrap()
                .history_peer_id,
            777
        );
        assert_eq!(current_item.external_id, migrated_item.external_id);
    }

    #[test]
    fn parses_photo_message_metadata() {
        let mut message = raw_message(43);
        message.media = Some(
            tl::types::MessageMediaPhoto {
                spoiler: false,
                live_photo: false,
                photo: Some(
                    tl::types::Photo {
                        has_stickers: false,
                        id: 100,
                        access_hash: 0,
                        file_reference: Vec::new(),
                        date: 1,
                        sizes: vec![
                            tl::types::PhotoSize {
                                r#type: "m".to_string(),
                                w: 320,
                                h: 240,
                                size: 123,
                            }
                            .into(),
                            tl::types::PhotoSize {
                                r#type: "x".to_string(),
                                w: 1280,
                                h: 720,
                                size: 456,
                            }
                            .into(),
                        ],
                        video_sizes: None,
                        dc_id: 2,
                    }
                    .into(),
                ),
                ttl_seconds: None,
                video: None,
            }
            .into(),
        );

        let item = parse_raw_message(&None, message)
            .expect("parse raw message")
            .expect("message item");
        let media = item.payload.media.expect("photo media");

        assert_eq!(item.payload.content_kind, "media_only");
        assert_eq!(media.kind, "photo");
        assert_eq!(media.metadata.summary.as_deref(), Some("Photo"));
        assert_eq!(media.metadata.width, Some(1280));
        assert_eq!(media.metadata.height, Some(720));
        assert_eq!(media.metadata.size_bytes, Some(456));
    }

    #[test]
    fn parses_document_media_kind_filename_and_dimensions() {
        let mut message = raw_message(44);
        message.message = "caption".to_string();
        message.media = Some(
            tl::types::MessageMediaDocument {
                nopremium: false,
                spoiler: false,
                video: false,
                round: false,
                voice: false,
                document: Some(
                    tl::types::Document {
                        id: 200,
                        access_hash: 0,
                        file_reference: Vec::new(),
                        date: 1,
                        mime_type: "image/png".to_string(),
                        size: 2048,
                        thumbs: None,
                        video_thumbs: None,
                        dc_id: 2,
                        attributes: vec![
                            tl::types::DocumentAttributeFilename {
                                file_name: "image.png".to_string(),
                            }
                            .into(),
                            tl::types::DocumentAttributeImageSize { w: 640, h: 480 }.into(),
                        ],
                    }
                    .into(),
                ),
                alt_documents: None,
                video_cover: None,
                video_timestamp: None,
                ttl_seconds: None,
            }
            .into(),
        );

        let item = parse_raw_message(&None, message)
            .expect("parse raw message")
            .expect("message item");
        let media = item.payload.media.expect("document media");

        assert_eq!(item.payload.content_kind, "text_with_media");
        assert_eq!(media.kind, "image");
        assert_eq!(media.metadata.file_name.as_deref(), Some("image.png"));
        assert_eq!(media.metadata.mime_type.as_deref(), Some("image/png"));
        assert_eq!(media.metadata.size_bytes, Some(2048));
        assert_eq!(media.metadata.width, Some(640));
        assert_eq!(media.metadata.height, Some(480));
    }

    #[test]
    fn skips_empty_raw_messages() {
        let item = parse_raw_message(&None, raw_message(45)).expect("parse raw message");
        assert!(item.is_none());
    }
}
