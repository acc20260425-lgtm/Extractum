type SettingsModel = {
  model: string;
  display_name: string;
};

type PersistedSettingsProfile = {
  profile_id: string;
  provider: string;
  default_model: string;
  api_key_configured: boolean;
  base_url: string;
};

export function filterSettingsModels<T extends SettingsModel>(models: T[], query: string): T[] {
  const normalized = query.trim().toLocaleLowerCase();
  if (!normalized) return models;
  return models.filter((model) =>
    `${model.display_name} ${model.model}`.toLocaleLowerCase().includes(normalized),
  );
}

export function materializeSettingsProfile(profile: PersistedSettingsProfile) {
  return {
    profileId: profile.profile_id,
    provider: profile.provider,
    defaultModel: profile.default_model,
    apiKeyConfigured: profile.api_key_configured,
    baseUrl: profile.base_url,
  };
}

export function settingsProfileStatus(activeProfile: string, selectedProfile: string, creating: boolean) {
  const editing = creating ? "new profile" : selectedProfile;
  return {
    activeLabel: `Active profile: ${activeProfile || "none"}`,
    editingLabel: `Editing profile: ${editing}`,
    showSetActiveAfterSave: !creating && selectedProfile !== activeProfile,
  };
}
