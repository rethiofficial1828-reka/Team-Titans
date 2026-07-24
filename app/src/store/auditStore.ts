import { create } from "zustand";

interface AuditEntry {
  id: number;
  timestamp: string;
  user: string | null;
  action: string;
  detail: string | null;
  chain_hash: string;
}

interface AuditStore {
  entries: AuditEntry[];
  loading: boolean;
  chainIntact: boolean | null;
  afterId: number | null;
  setEntries: (entries: AuditEntry[]) => void;
  appendEntries: (entries: AuditEntry[]) => void;
  setLoading: (loading: boolean) => void;
  setChainIntact: (intact: boolean, mismatchAt?: number) => void;
  setAfterId: (id: number | null) => void;
}

export const useAuditStore = create<AuditStore>((set) => ({
  entries: [],
  loading: false,
  chainIntact: null,
  afterId: null,
  setEntries: (entries) => set({ entries }),
  appendEntries: (entries) =>
    set((s) => ({ entries: [...s.entries, ...entries] })),
  setLoading: (loading) => set({ loading }),
  setChainIntact: (chainIntact) => set({ chainIntact }),
  setAfterId: (afterId) => set({ afterId }),
}));
