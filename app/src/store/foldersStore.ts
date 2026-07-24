import { create } from "zustand";

interface ProtectedItem {
  id: number;
  path: string;
  state: "Idle" | "Protecting" | "Protected" | "Unprotecting" | "CrashRecoveryPending";
  protected_at: string | null;
  protected_by: string | null;
  files_processed: number;
  files_total: number | null;
}

interface FoldersStore {
  items: ProtectedItem[];
  loading: boolean;
  setItems: (items: ProtectedItem[]) => void;
  setLoading: (loading: boolean) => void;
  updateItemState: (id: number, state: ProtectedItem["state"]) => void;
}

export const useFoldersStore = create<FoldersStore>((set) => ({
  items: [],
  loading: false,
  setItems: (items) => set({ items }),
  setLoading: (loading) => set({ loading }),
  updateItemState: (id, state) =>
    set((s) => ({
      items: s.items.map((item) => (item.id === id ? { ...item, state } : item)),
    })),
}));
