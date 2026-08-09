import YoutubeSettingsPanel from "$lib/components/settings/youtube-settings-panel.svelte";

export const ACCOUNT_SOURCE_ACCESS = Object.freeze({
  eyebrow: "Source access",
  pageTitle: "Accounts",
  description: "Manage source identities and authentication used for sync and analysis.",
  sections: Object.freeze([
    Object.freeze({
      kind: "telegram",
      heading: "Telegram accounts",
      shell: "desk-panel account-catalog",
      labelledBy: "telegram-accounts-heading",
    }),
    Object.freeze({
      kind: "youtube",
      heading: "YouTube access",
      shell: "desk-panel youtube-access-shell",
      labelledBy: "youtube-access-heading",
      panel: YoutubeSettingsPanel,
      panelProps: Object.freeze({ embedded: true }),
    }),
  ] as const),
});

export function sourceAccessNavigationItem() {
  return Object.freeze({ href: "/accounts", label: "Accounts", caption: "Source access" });
}
