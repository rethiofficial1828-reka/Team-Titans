import { useState, useEffect } from "react";
import { Lock, AlertTriangle } from "lucide-react";

export default function PasswordModal({
  title, subtitle, onConfirm, onCancel, adminPassword,
}: {
  title: string; subtitle: string;
  onConfirm: () => void; onCancel: () => void;
  adminPassword: string;
}) {
  const [pw, setPw] = useState("");
  const [err, setErr] = useState<string | null>(null);
  const [isOpen, setIsOpen] = useState(false);
  const [isClosing, setIsClosing] = useState(false);

  useEffect(() => {
    const t = setTimeout(() => setIsOpen(true), 10);
    return () => clearTimeout(t);
  }, []);

  const triggerClose = (callback: () => void) => {
    setIsClosing(true);
    setTimeout(callback, 150);
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (pw !== adminPassword) { setErr("Invalid administrator credentials."); return; }
    triggerClose(onConfirm);
  };

  return (
    <div className={`t-modal-overlay ${isOpen && !isClosing ? 'is-open' : ''} ${isClosing ? 'is-closing' : ''}`} style={{
      position: "fixed", inset: 0, zIndex: 100,
      background: "rgba(5,10,18,.85)", backdropFilter: "blur(6px)",
      display: "flex", alignItems: "center", justifyContent: "center", padding: 24,
    }}>
      <div className={`t-modal ${isOpen && !isClosing ? 'is-open' : ''} ${isClosing ? 'is-closing' : ''}`} style={{
        width: "100%", maxWidth: 420,
        background: "var(--panel)", border: "1.5px solid var(--hair)",
        borderRadius: 14, padding: 28, boxShadow: "0 24px 60px rgba(0,0,0,.6)",
      }}>
        <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 20 }}>
          <div style={{ padding: 10, background: "var(--void)", border: "1.5px solid var(--hair2)", borderRadius: 10 }}>
            <Lock size={20} color="var(--amber)" />
          </div>
          <div>
            <div style={{ fontSize: 13, fontWeight: 700, color: "var(--hi)", marginBottom: 2 }}>{title}</div>
            <div style={{ fontSize: 11, color: "var(--lo)" }}>{subtitle}</div>
          </div>
        </div>
        <form onSubmit={handleSubmit} style={{ display: "flex", flexDirection: "column", gap: 14 }}>
          <div>
            <div className="section-header" style={{ marginBottom: 8 }}>Master passphrase</div>
            <input type="password" value={pw} onChange={e => setPw(e.target.value)} className="qfs-input" placeholder="            " required />
          </div>
          {err && (
            <div style={{ display: "flex", alignItems: "center", gap: 8, padding: "10px 14px", background: "rgba(229,82,90,.1)", border: "1px solid rgba(229,82,90,.25)", borderRadius: 8 }}>
              <AlertTriangle size={14} color="var(--red)" />
              <span style={{ fontSize: 12, color: "var(--red)" }}>{err}</span>
            </div>
          )}
          <div style={{ display: "flex", gap: 10, justifyContent: "flex-end", marginTop: 4 }}>
            <button type="button" onClick={() => triggerClose(onCancel)} className="btn-ghost">Cancel</button>
            <button type="submit" className="btn-teal">Confirm</button>
          </div>
        </form>
      </div>
    </div>
  );
}
