export default function StrengthMeter({ pass }: { pass: string }) {
  if (!pass) return null;
  let s = 0;
  if (pass.length >= 8)  s++;
  if (pass.length >= 12) s++;
  if (/[A-Z]/.test(pass) && /[a-z]/.test(pass)) s++;
  if (/\d/.test(pass)) s++;
  if (/[^A-Za-z0-9]/.test(pass)) s++;

  const tiers = [
    { label: "Insufficient", color: "var(--red)" },
    { label: "Weak",         color: "var(--red)" },
    { label: "Fair",         color: "var(--amber)" },
    { label: "Good",         color: "var(--teal)" },
    { label: "Strong",       color: "var(--teal)" },
    { label: "Enterprise",   color: "var(--green)" },
  ];
  const t = tiers[Math.min(s, 5)];

  return (
    <div style={{ marginTop: 10 }}>
      <div style={{ display: "flex", gap: 4, height: 4, marginBottom: 8 }}>
        {[0,1,2,3,4].map(i => (
          <div key={i} style={{
            flex: 1, borderRadius: 2,
            background: i < s ? t.color : "var(--line-lo)",
            transition: "all .3s"
          }} />
        ))}
      </div>
      <div style={{ display: "flex", justifyContent: "space-between", fontSize: 11, fontWeight: 600 }}>
        <span style={{ color: "var(--lo2)" }}>Security Grade</span>
        <span style={{ color: t.color }}>{t.label}</span>
      </div>
    </div>
  );
}
