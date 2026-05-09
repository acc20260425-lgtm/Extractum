import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  clearYoutubeAuth,
  getYoutubeAuthStatus,
  getYoutubeSettings,
  saveYoutubeCookies,
  saveYoutubeSettings,
} from "./youtube-settings";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

describe("youtube settings API", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("loads youtube settings", async () => {
    invokeMock.mockResolvedValueOnce({
      authEnabled: false,
      preferredCaptionsLanguage: "original",
      delayBetweenRequestsMs: 1000,
      maxParallelVideoSyncs: 1,
      maxParallelCommentSyncs: 1,
      pauseOnAuthChallenge: true,
      dailySoftLimit: 0,
      retryBackoffMs: 3000,
      stopAfterConsecutiveFailures: 3,
    });

    await expect(getYoutubeSettings()).resolves.toMatchObject({
      authEnabled: false,
      delayBetweenRequestsMs: 1000,
    });
    expect(invokeMock).toHaveBeenLastCalledWith("get_youtube_settings");
  });

  it("saves youtube settings with a settings argument", async () => {
    const settings = {
      authEnabled: true,
      preferredCaptionsLanguage: "en",
      delayBetweenRequestsMs: 500,
      maxParallelVideoSyncs: 2,
      maxParallelCommentSyncs: 1,
      pauseOnAuthChallenge: true,
      dailySoftLimit: 200,
      retryBackoffMs: 5000,
      stopAfterConsecutiveFailures: 4,
    };
    invokeMock.mockResolvedValueOnce(settings);

    await expect(saveYoutubeSettings(settings)).resolves.toMatchObject(settings);
    expect(invokeMock).toHaveBeenLastCalledWith("save_youtube_settings", {
      settings,
    });
  });

  it("reads auth status without exposing cookies", async () => {
    invokeMock.mockResolvedValueOnce({
      enabled: true,
      hasCookies: true,
      message: "Cookies stored",
    });

    await expect(getYoutubeAuthStatus()).resolves.toMatchObject({
      enabled: true,
      hasCookies: true,
    });
    expect(invokeMock).toHaveBeenLastCalledWith("get_youtube_auth_status");
  });

  it("saves and clears youtube cookies through dedicated commands", async () => {
    invokeMock.mockResolvedValueOnce({
      enabled: true,
      hasCookies: true,
      message: "Cookies stored",
    });

    await saveYoutubeCookies("cookie text");
    expect(invokeMock).toHaveBeenLastCalledWith("save_youtube_cookies", {
      cookies: "cookie text",
    });

    invokeMock.mockResolvedValueOnce({
      enabled: false,
      hasCookies: false,
      message: "Auth disabled",
    });

    await clearYoutubeAuth();
    expect(invokeMock).toHaveBeenLastCalledWith("clear_youtube_auth");
  });
});
