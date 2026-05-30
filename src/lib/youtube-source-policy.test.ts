import { describe, expect, it } from "vitest";
import { isRetryableYoutubeAvailabilityStatus } from "./youtube-source-policy";

describe("youtube source policy", () => {
  it("classifies retryable YouTube availability statuses", () => {
    expect(isRetryableYoutubeAvailabilityStatus("live_ended_transcript_pending")).toBe(true);
    expect(isRetryableYoutubeAvailabilityStatus("no_captions")).toBe(true);
    expect(isRetryableYoutubeAvailabilityStatus("unavailable_unknown")).toBe(true);
    expect(isRetryableYoutubeAvailabilityStatus("available")).toBe(false);
    expect(isRetryableYoutubeAvailabilityStatus("private_or_auth_required")).toBe(false);
    expect(isRetryableYoutubeAvailabilityStatus(null)).toBe(false);
    expect(isRetryableYoutubeAvailabilityStatus(undefined)).toBe(false);
  });
});
