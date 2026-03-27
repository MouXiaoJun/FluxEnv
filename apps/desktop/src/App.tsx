import { useCallback, useEffect, useState } from "react";

type ProviderItem = {
  name: string;
  envKey: string;
  envValue: string;
  enabled: boolean;
  builtin: boolean;
};

type Lang = "zh" | "en";

type EditingDraft = {
  name: string;
  envKey: string;
  envValue: string;
};

const i18n: Record<Lang, Record<string, string>> = {
  zh: {
    title: "FluxEnv",
    subtitle: "跨平台环境管理器，支持配置切换与密钥管理。",
    run: "使用当前配置运行",
    secret: "查看密钥状态",
    addPlaceholder: "新增服务商，例如 moonshot",
    addProvider: "添加服务商",
    statusReady: "会话模式已就绪。",
    tauriUnavailable: "当前不在 Tauri 窗口，请在桌面应用中运行。",
    runFailed: "运行失败",
    statusFailed: "状态检查失败",
    toggleFailed: "开关失败",
    addEmpty: "请输入服务商名称。",
    addExists: "该服务商已存在。",
    addDone: "服务商已添加（当前为本地会话）。",
    removeDone: "服务商已删除。",
    custom: "自定义",
    on: "开启",
    off: "关闭",
    enable: "启用",
    disable: "停用",
    delete: "删除",
    edit: "编辑",
    save: "保存",
    cancel: "取消",
    language: "语言",
    noEditName: "服务商名称不能为空。",
    noEditKey: "环境变量 Key 不能为空。",
    saveDone: "服务商配置已保存，并已同步到会话后端。",
    localToggle: "本地服务商状态已切换。",
    checkUpdate: "检查更新",
    updateChecking: "正在检查更新…",
    updateLatest: "当前已是最新版本。",
    updateFound: "发现新版本",
    updateInstalling: "正在下载并安装更新…",
    updateRelaunch: "更新完成，正在重启…",
    updateError: "更新失败",
    clearSecret: "清除密钥",
    secretStored: "已保存到系统钥匙串",
    secretPlaceholder: "输入新密钥（留空保存则不改）",
    secretCleared: "已清除本地钥匙串中的该项"
  },
  en: {
    title: "FluxEnv",
    subtitle: "Cross-platform environment manager for profile switching and secure secrets.",
    run: "Run with Selected Profile",
    secret: "View Secret Status",
    addPlaceholder: "add provider name, e.g. moonshot",
    addProvider: "Add Provider",
    statusReady: "Session mode is ready.",
    tauriUnavailable: "Tauri invoke is unavailable. Please run in Tauri window.",
    runFailed: "Run failed",
    statusFailed: "Status check failed",
    toggleFailed: "Toggle failed",
    addEmpty: "Please enter a provider name.",
    addExists: "Provider already exists.",
    addDone: "Provider added (local session for now).",
    removeDone: "Provider removed.",
    custom: "custom",
    on: "ON",
    off: "OFF",
    enable: "Enable",
    disable: "Disable",
    delete: "Delete",
    edit: "Edit",
    save: "Save",
    cancel: "Cancel",
    language: "Language",
    noEditName: "Provider name is required.",
    noEditKey: "Env key is required.",
    saveDone: "Provider configuration saved and synced to session backend.",
    localToggle: "Local provider state changed.",
    checkUpdate: "Check for updates",
    updateChecking: "Checking for updates…",
    updateLatest: "You are on the latest version.",
    updateFound: "Update available",
    updateInstalling: "Downloading and installing…",
    updateRelaunch: "Update installed, restarting…",
    updateError: "Update failed",
    clearSecret: "Clear secret",
    secretStored: "Stored in OS keychain",
    secretPlaceholder: "New API key (leave empty to keep)",
    secretCleared: "Removed from OS keychain"
  }
};

function App() {
  const defaultLang: Lang =
    typeof navigator !== "undefined" && navigator.language.toLowerCase().startsWith("zh")
      ? "zh"
      : "en";
  const [lang, setLang] = useState<Lang>(defaultLang);
  const t = (key: string) => i18n[lang][key] ?? key;

  const [status, setStatus] = useState(t("statusReady"));
  const [providers, setProviders] = useState<ProviderItem[]>([
    { name: "openai", envKey: "OPENAI_API_KEY", envValue: "", enabled: false, builtin: true },
    { name: "anthropic", envKey: "ANTHROPIC_API_KEY", envValue: "", enabled: false, builtin: true },
    { name: "deepseek", envKey: "DEEPSEEK_API_KEY", envValue: "", enabled: false, builtin: true },
    { name: "openrouter", envKey: "OPENROUTER_API_KEY", envValue: "", enabled: false, builtin: true }
  ]);
  const [draftName, setDraftName] = useState("");
  const [editingName, setEditingName] = useState<string | null>(null);
  const [editingDraft, setEditingDraft] = useState<EditingDraft>({
    name: "",
    envKey: "",
    envValue: ""
  });
  const invoke = (window as any).__TAURI_INTERNALS__?.invoke as
    | ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>)
    | undefined;

  const [secretStoredMap, setSecretStoredMap] = useState<Record<string, boolean>>({});

  const secretFlagKey = (name: string, envKey: string) => `${name}\n${envKey}`;

  const refreshSecretFlags = useCallback(
    async (list: ProviderItem[]) => {
      if (!invoke) return;
      const next: Record<string, boolean> = {};
      await Promise.all(
        list.map(async (p) => {
          try {
            const stored = await invoke("secret_is_stored", {
              providerName: p.name,
              envKey: p.envKey
            });
            next[secretFlagKey(p.name, p.envKey)] = Boolean(stored);
          } catch {
            next[secretFlagKey(p.name, p.envKey)] = false;
          }
        })
      );
      setSecretStoredMap(next);
    },
    [invoke]
  );

  useEffect(() => {
    void refreshSecretFlags(providers);
  }, [providers, refreshSecretFlags]);

  const handleRunClick = async () => {
    if (!invoke) {
      setStatus(t("tauriUnavailable"));
      return;
    }
    try {
      const result = await invoke("run_with_selected_profile", { profile: "dev" });
      setStatus(String(result));
    } catch (error) {
      setStatus(`${t("runFailed")}: ${String(error)}`);
    }
  };

  const handleSecretClick = async () => {
    if (!invoke) {
      setStatus(t("tauriUnavailable"));
      return;
    }
    try {
      const result = await invoke("get_secret_status");
      setStatus(String(result));
    } catch (error) {
      setStatus(`${t("statusFailed")}: ${String(error)}`);
    }
  };

  const handleCheckUpdate = async () => {
    if (!invoke) {
      setStatus(t("tauriUnavailable"));
      return;
    }
    try {
      setStatus(t("updateChecking"));
      const { check } = await import("@tauri-apps/plugin-updater");
      const { relaunch } = await import("@tauri-apps/plugin-process");
      const update = await check();
      if (!update) {
        setStatus(t("updateLatest"));
        return;
      }
      setStatus(`${t("updateFound")}: ${update.version}`);
      setStatus(t("updateInstalling"));
      await update.downloadAndInstall();
      setStatus(t("updateRelaunch"));
      await relaunch();
    } catch (error) {
      setStatus(`${t("updateError")}: ${String(error)}`);
    }
  };

  const handleToggleProvider = async (providerName: string) => {
    if (!invoke) {
      setStatus(t("tauriUnavailable"));
      return;
    }
    const current = providers.find((p) => p.name === providerName);
    if (!current) return;
    const next = !current.enabled;
    try {
      if (current.builtin) {
        const result = await invoke("toggle_provider", {
          providerName,
          enabled: next
        });
        setStatus(String(result));
      } else {
        const result = await invoke("upsert_provider", {
          originalName: providerName,
          name: providerName,
          envKey: current.envKey,
          envValue: current.envValue,
          enabled: next,
          writeSecret: false
        });
        setStatus(String(result));
      }
      setProviders((prev) =>
        prev.map((p) => (p.name === providerName ? { ...p, enabled: next } : p))
      );
    } catch (error) {
      setStatus(`${t("toggleFailed")}: ${String(error)}`);
    }
  };

  const handleAddProvider = async () => {
    const normalized = draftName.trim().toLowerCase();
    if (!normalized) {
      setStatus(t("addEmpty"));
      return;
    }
    if (providers.some((p) => p.name === normalized)) {
      setStatus(`${t("addExists")} (${normalized})`);
      return;
    }
    const envKey = `${normalized.toUpperCase()}_API_KEY`;
    if (invoke) {
      try {
        await invoke("upsert_provider", {
          originalName: null,
          name: normalized,
          envKey,
          envValue: "",
          enabled: false,
          writeSecret: false
        });
      } catch (error) {
        setStatus(`${t("runFailed")}: ${String(error)}`);
        return;
      }
    }
    setProviders((prev) => [
      ...prev,
      {
        name: normalized,
        envKey,
        envValue: "",
        enabled: false,
        builtin: false
      }
    ]);
    setDraftName("");
    setStatus(`${t("addDone")} (${normalized})`);
  };

  const handleDeleteProvider = async (providerName: string) => {
    if (!invoke) {
      setStatus(t("tauriUnavailable"));
      return;
    }
    try {
      await invoke("remove_provider", { name: providerName });
      setProviders((prev) => prev.filter((p) => p.name !== providerName));
      setStatus(`${t("removeDone")} (${providerName})`);
    } catch (error) {
      setStatus(`${t("runFailed")}: ${String(error)}`);
    }
  };

  const startEditing = (provider: ProviderItem) => {
    setEditingName(provider.name);
    setEditingDraft({
      name: provider.name,
      envKey: provider.envKey,
      envValue: provider.envValue
    });
  };

  const cancelEditing = () => {
    setEditingName(null);
  };

  const handleClearEditingSecret = async () => {
    if (!invoke || !editingName) {
      setStatus(t("tauriUnavailable"));
      return;
    }
    const live = providers.find((p) => p.name === editingName);
    if (!live) return;
    try {
      await invoke("clear_provider_secret", {
        providerName: live.name,
        envKey: live.envKey
      });
      setEditingDraft((d) => ({ ...d, envValue: "" }));
      setStatus(t("secretCleared"));
      await refreshSecretFlags(providers);
    } catch (error) {
      setStatus(`${t("runFailed")}: ${String(error)}`);
    }
  };

  const saveEditing = async (originalName: string) => {
    const nextName = editingDraft.name.trim().toLowerCase();
    const nextKey = editingDraft.envKey.trim().toUpperCase();
    if (!nextName) {
      setStatus(t("noEditName"));
      return;
    }
    if (!nextKey) {
      setStatus(t("noEditKey"));
      return;
    }
    if (nextName !== originalName && providers.some((p) => p.name === nextName)) {
      setStatus(`${t("addExists")} (${nextName})`);
      return;
    }
    const current = providers.find((p) => p.name === originalName);
    const enabled = current?.enabled ?? false;
    if (!invoke) {
      setStatus(t("tauriUnavailable"));
      return;
    }
    try {
      const trimmed = editingDraft.envValue.trim();
      const writeSecret = trimmed.length > 0;
      const result = await invoke("upsert_provider", {
        originalName,
        name: nextName,
        envKey: nextKey,
        envValue: trimmed,
        enabled,
        writeSecret
      });
      setProviders((prev) =>
        prev.map((p) =>
          p.name === originalName
            ? { ...p, name: nextName, envKey: nextKey, envValue: "" }
            : p
        )
      );
      setEditingName(null);
      setStatus(String(result));
    } catch (error) {
      setStatus(`${t("runFailed")}: ${String(error)}`);
    }
  };

  return (
    <main className="app">
      <section className="card">
        <div className="toolbar">
          <div className="brand">
            <img
              alt=""
              className="brand-logo"
              decoding="async"
              height={40}
              src="/fluxenv-icon.svg"
              width={40}
            />
            <div>
              <h1 className="title">{t("title")}</h1>
              <p className="muted">{t("subtitle")}</p>
            </div>
          </div>
          <div className="lang-switch">
            <span>{t("language")}</span>
            <button className="btn" onClick={() => setLang("zh")} type="button">
              中文
            </button>
            <button className="btn" onClick={() => setLang("en")} type="button">
              EN
            </button>
          </div>
        </div>
        <div className="actions">
          <button className="btn btn-primary" type="button" onClick={handleRunClick}>
            {t("run")}
          </button>
          <button className="btn btn-security" type="button" onClick={handleSecretClick}>
            {t("secret")}
          </button>
          <button className="btn" type="button" onClick={handleCheckUpdate}>
            {t("checkUpdate")}
          </button>
        </div>
        <div className="provider-create">
          <input
            className="provider-input"
            onChange={(e) => setDraftName(e.target.value)}
            placeholder={t("addPlaceholder")}
            type="text"
            value={draftName}
          />
          <button className="btn" onClick={handleAddProvider} type="button">
            {t("addProvider")}
          </button>
        </div>
        <div className="provider-list">
          {providers.map((provider) => (
            <div className="provider-row" key={provider.name}>
              {editingName === provider.name ? (
                <div className="provider-editor">
                  <input
                    className="provider-input"
                    onChange={(e) => setEditingDraft((d) => ({ ...d, name: e.target.value }))}
                    value={editingDraft.name}
                  />
                  <input
                    className="provider-input"
                    onChange={(e) => setEditingDraft((d) => ({ ...d, envKey: e.target.value }))}
                    value={editingDraft.envKey}
                  />
                  {secretStoredMap[secretFlagKey(provider.name, provider.envKey)] ? (
                    <span className="secret-badge">{t("secretStored")}</span>
                  ) : null}
                  <input
                    className="provider-input"
                    onChange={(e) => setEditingDraft((d) => ({ ...d, envValue: e.target.value }))}
                    placeholder={t("secretPlaceholder")}
                    type="password"
                    autoComplete="off"
                    value={editingDraft.envValue}
                  />
                  <div className="provider-actions">
                    <button className="btn" onClick={() => saveEditing(provider.name)} type="button">
                      {t("save")}
                    </button>
                    <button className="btn" onClick={handleClearEditingSecret} type="button">
                      {t("clearSecret")}
                    </button>
                    <button className="btn" onClick={cancelEditing} type="button">
                      {t("cancel")}
                    </button>
                  </div>
                </div>
              ) : (
                <>
                  <div className="provider-meta">
                    <strong>{provider.name}</strong>
                    <span className={provider.enabled ? "provider-on" : "provider-off"}>
                      {provider.enabled ? t("on") : t("off")}
                    </span>
                    {!provider.builtin && <span className="provider-tag">{t("custom")}</span>}
                    {secretStoredMap[secretFlagKey(provider.name, provider.envKey)] ? (
                      <span className="secret-badge">{t("secretStored")}</span>
                    ) : null}
                    <code className="provider-key">{provider.envKey}</code>
                  </div>
                  <div className="provider-actions">
                    <button className="btn" onClick={() => handleToggleProvider(provider.name)} type="button">
                      {provider.enabled ? t("disable") : t("enable")}
                    </button>
                    <button className="btn" onClick={() => startEditing(provider)} type="button">
                      {t("edit")}
                    </button>
                    <button
                      className="btn btn-danger"
                      disabled={provider.builtin}
                      onClick={() => handleDeleteProvider(provider.name)}
                      type="button"
                    >
                      {t("delete")}
                    </button>
                  </div>
                </>
              )}
            </div>
          ))}
        </div>
        <p className="status">{status}</p>
      </section>
    </main>
  );
}

export default App;
