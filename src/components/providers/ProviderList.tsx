
import { useEffect, useState } from "react";
import { ShieldCheck } from "lucide-react";
import { toast } from "sonner";
import { EmptyState } from "@/components/shared/EmptyState";
import { createProvider, deleteProvider, listProviders, updateProvider } from "@/lib/tauri";
import type { Provider } from "@/lib/types";
import { ProviderHeader } from "./components/ProviderHeader";
import { ProviderCard } from "./components/ProviderCard";
import { ProviderDialog } from "./components/ProviderDialog";
import { DeleteDialog } from "./components/DeleteDialog";

export function ProviderList() {
  const [providers, setProviders] = useState<Provider[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [editProvider, setEditProvider] = useState<Provider | null>(null);
  const [pendingDelete, setPendingDelete] = useState<Provider | null>(null);
  const [togglingId, setTogglingId] = useState<string | null>(null);
  const [form, setForm] = useState({
    name: "",
    base_url: "",
    api_key: "",
    protocol: "anthropic" as "anthropic" | "openai",
    model_mapping: "",
    auth_header: "x-api-key" as "x-api-key" | "bearer",
    keyword: "sonnet",
    enabled: false,
  });

  const refresh = () => listProviders().then(setProviders).catch(console.error);

  useEffect(() => {
    refresh();
  }, []);

  const openCreate = () => {
    setForm({
      name: "",
      base_url: "",
      api_key: "",
      protocol: "anthropic",
      model_mapping: "",
      auth_header: "x-api-key",
      keyword: "sonnet",
      enabled: false,
    });
    setEditProvider(null);
    setShowForm(true);
  };

  const openEdit = (provider: Provider) => {
    setForm({
      name: provider.name,
      base_url: provider.base_url,
      api_key: "",
      protocol: provider.protocol,
      model_mapping: provider.model_mapping || "",
      auth_header: provider.auth_header,
      keyword: provider.keyword,
      enabled: provider.enabled,
    });
    setEditProvider(provider);
    setShowForm(true);
  };

  const canSaveProvider =
    form.name.trim().length > 0 &&
    form.base_url.trim().length > 0 &&
    form.keyword.trim().length > 0 &&
    (Boolean(editProvider) || form.api_key.trim().length > 0);

  const handleSave = async () => {
    if (!canSaveProvider) {
      toast.error("请填写必填字段");
      return;
    }

    try {
      if (editProvider) {
        await updateProvider({
          id: editProvider.id,
          name: form.name,
          base_url: form.base_url,
          api_key: form.api_key || undefined,
          protocol: form.protocol,
          model_mapping: form.model_mapping || undefined,
          auth_header: form.auth_header,
          keyword: form.keyword,
          enabled: form.enabled,
        });
        toast.success("提供商已更新");
      } else {
        await createProvider({
          name: form.name,
          base_url: form.base_url,
          api_key: form.api_key,
          protocol: form.protocol,
          model_mapping: form.model_mapping || undefined,
          auth_header: form.auth_header,
          keyword: form.keyword,
          enabled: form.enabled,
        });
        toast.success("提供商已创建");
      }
      setShowForm(false);
      refresh();
    } catch (error) {
      toast.error(`保存失败: ${error}`);
    }
  };

  const handleDelete = async () => {
    if (!pendingDelete) return;
    try {
      await deleteProvider(pendingDelete.id);
      toast.success("提供商已删除");
      setPendingDelete(null);
      refresh();
    } catch (error) {
      toast.error(`删除失败: ${error}`);
    }
  };

  const handleToggle = async (provider: Provider) => {
    if (togglingId) return;
    setTogglingId(provider.id);
    try {
      await updateProvider({
        id: provider.id,
        name: provider.name,
        base_url: provider.base_url,
        protocol: provider.protocol,
        model_mapping: provider.model_mapping || undefined,
        auth_header: provider.auth_header,
        keyword: provider.keyword,
        enabled: !provider.enabled,
      });
      toast.success(`提供商已${!provider.enabled ? "启用" : "禁用"}`);
      refresh();
    } catch (error) {
      toast.error(`切换失败: ${error}`);
    } finally {
      setTogglingId(null);
    }
  };

  if (providers.length === 0 && !showForm) {
    return (
      <div className="flex flex-col gap-6 fade-in">
        <ProviderHeader providersCount={0} enabledCount={0} onAddClick={openCreate} />
        <EmptyState
          icon={<ShieldCheck className="h-10 w-10" />}
          title="暂无提供商"
          description="添加您的第一个提供商开始使用"
        />
        <ProviderDialog
          editProvider={editProvider}
          form={form}
          onChange={setForm}
          onClose={() => setShowForm(false)}
          onSave={handleSave}
          open={showForm}
          canSave={canSaveProvider}
        />
      </div>
    );
  }

  const enabledCount = providers.filter((p) => p.enabled).length;

  return (
    <div className="flex flex-col gap-6 fade-in">
      <ProviderHeader providersCount={providers.length} enabledCount={enabledCount} onAddClick={openCreate} />

      <div className="flex flex-col gap-4">
        {providers.map((provider) => (
          <ProviderCard
            key={provider.id}
            provider={provider}
            togglingId={togglingId}
            onToggle={handleToggle}
            onEdit={openEdit}
            onDelete={() => setPendingDelete(provider)}
          />
        ))}
      </div>

      <ProviderDialog
        editProvider={editProvider}
        form={form}
        onChange={setForm}
        onClose={() => setShowForm(false)}
        onSave={handleSave}
        open={showForm}
        canSave={canSaveProvider}
      />
      <DeleteDialog
        onClose={() => setPendingDelete(null)}
        onConfirm={handleDelete}
        provider={pendingDelete}
      />
    </div>
  );
}
