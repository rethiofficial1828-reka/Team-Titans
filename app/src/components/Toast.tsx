import { CheckCircle, AlertTriangle } from "lucide-react";

export default function Toast({ msg, type }: { msg: string; type: "success" | "error" }) {
  return (
    <div style={{
      position: "fixed", top: 20, right: 20, zIndex: 200,
      padding: "12px 20px", borderRadius: 10,
      background: type === "success" ? "rgba(79,216,122,.15)" : "rgba(229,82,90,.15)",
      border: `1.5px solid ${type === "success" ? "rgba(79,216,122,.4)" : "rgba(229,82,90,.4)"}`,
      color: type === "success" ? "var(--green)" : "var(--red)",
      fontSize: 12, fontWeight: 600, fontFamily: "'IBM Plex Mono', monospace",
      boxShadow: "0 8px 32px rgba(0,0,0,.4)",
      animation: "toast-in .3s ease both",
      maxWidth: 400,
    }}>
      <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
        {type === "success" ? <CheckCircle size={14} /> : <AlertTriangle size={14} />}
        {msg}
      </div>
    </div>
  );
}
