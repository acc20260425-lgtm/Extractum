use std::future::Future;

use extractum_core::error::{AppError, AppResult};
use grammers_client::peer::Peer;
use tokio::time::{timeout, Duration};

const TELEGRAM_SOURCE_PHOTO_TIMEOUT_MS: u64 = 750;

pub(super) async fn peer_photo_bytes_with_timeout(
    client: &grammers_client::Client,
    peer: &Peer,
) -> Option<Vec<u8>> {
    best_effort_avatar_with_timeout(
        TELEGRAM_SOURCE_PHOTO_TIMEOUT_MS,
        peer_photo_bytes(client, peer),
    )
    .await
}

async fn best_effort_avatar_with_timeout<F>(timeout_ms: u64, load: F) -> Option<Vec<u8>>
where
    F: Future<Output = AppResult<Option<Vec<u8>>>>,
{
    timeout(Duration::from_millis(timeout_ms), load)
        .await
        .ok()
        .and_then(Result::ok)
        .flatten()
}

pub(super) async fn peer_photo_bytes(
    client: &grammers_client::Client,
    peer: &Peer,
) -> AppResult<Option<Vec<u8>>> {
    let Some(photo) = peer.photo(false).await else {
        return Ok(None);
    };

    let mut bytes = Vec::new();
    let mut download = client.iter_download(&photo).chunk_size(4 * 1024);
    while let Some(chunk) = download
        .next()
        .await
        .map_err(|error| AppError::network(error.to_string()))?
    {
        bytes.extend(chunk);
    }

    if bytes.is_empty() {
        return Ok(None);
    }

    Ok(Some(bytes))
}

#[cfg(test)]
mod tests {
    use extractum_core::error::AppError;
    use tokio::time::{sleep, Duration};

    use super::{best_effort_avatar_with_timeout, TELEGRAM_SOURCE_PHOTO_TIMEOUT_MS};

    #[tokio::test(start_paused = true)]
    async fn peer_photo_bytes_returns_owned_bytes_and_suppresses_timeout_and_transport_failure() {
        assert_eq!(TELEGRAM_SOURCE_PHOTO_TIMEOUT_MS, 750);

        let owned =
            best_effort_avatar_with_timeout(50, async { Ok(Some(vec![0x01, 0x02, 0x03])) }).await;
        assert_eq!(owned, Some(vec![0x01, 0x02, 0x03]));

        let timed_out = best_effort_avatar_with_timeout(1, async {
            sleep(Duration::from_millis(25)).await;
            Ok(Some(vec![0x04]))
        })
        .await;
        assert_eq!(timed_out, None);

        let transport_failed = best_effort_avatar_with_timeout(50, async {
            Err(AppError::network("deterministic avatar transport failure"))
        })
        .await;
        assert_eq!(transport_failed, None);
    }
}
