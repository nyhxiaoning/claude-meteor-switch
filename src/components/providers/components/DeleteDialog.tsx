
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import type { Provider } from "@/lib/types";

interface DeleteDialogProps {
  onClose: () => void;
  onConfirm: () => void;
  provider: Provider | null;
}

export function DeleteDialog({ onClose, onConfirm, provider }: DeleteDialogProps) {
  return (
    <Dialog open={Boolean(provider)} onOpenChange={(nextOpen) => { if (!nextOpen) onClose(); }}>
      <DialogContent className="signal-panel">
        <DialogHeader>
          <DialogTitle className="text-left font-semibold text-xl text-destructive">
            删除提供商
          </DialogTitle>
        </DialogHeader>
        <div className="space-y-4 mt-2">
          <p className="text-muted-foreground text-sm">
            确定要删除这个提供商吗？
          </p>
          {provider && (
            <div className="bg-destructive/5 border border-destructive/20 px-5 py-4">
              <p className="text-muted-foreground text-xs font-mono">提供商:</p>
              <p className="mt-2 text-foreground font-semibold">{provider.name}</p>
              <p className="mt-1 text-muted-foreground text-sm font-mono">{provider.base_url}</p>
            </div>
          )}
        </div>
        <DialogFooter className="flex gap-3 mt-6">
          <Button onClick={onClose} variant="outline" className="flex-1">
            取消
          </Button>
          <Button onClick={onConfirm} variant="destructive" className="flex-1">
            删除
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
