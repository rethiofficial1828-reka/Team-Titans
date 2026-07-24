import { create } from "zustand";
import type { Screen } from "../types";

interface SessionInfo {
  session_id: string;
  username: string;
  role: string;
  expires_at: string;
}

interface AuthStore {
  session: SessionInfo | null;
  currentScreen: Screen;
  setSession: (session: SessionInfo | null) => void;
  setScreen: (screen: Screen) => void;
  logout: () => void;
}

export const useAuthStore = create<AuthStore>((set) => ({
  session: null,
  currentScreen: "welcome",
  setSession: (session) =>
    set({ session, currentScreen: session ? "dashboard" : "login" }),
  setScreen: (currentScreen) => set({ currentScreen }),
  logout: () => set({ session: null, currentScreen: "login" }),
}));
