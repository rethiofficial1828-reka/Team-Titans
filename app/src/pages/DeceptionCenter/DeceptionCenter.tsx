import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Shield, Brain, Activity, Crosshair } from 'lucide-react';

interface DeceptionStatus {
  epsilon: number;
  total_episodes: number;
  q_table_size: number;
  latest_action: string | null;
}

export function DeceptionCenter() {
  const [status, setStatus] = useState<DeceptionStatus | null>(null);
  const [feed, setFeed] = useState<string[]>([]);
  const [depth, setDepth] = useState(1);
  const [density, setDensity] = useState(5);

  const fetchStatus = async () => {
    try {
      const st = await invoke<DeceptionStatus>("get_deception_status");
      setStatus(st);
    } catch (e) {
      console.error("Failed to fetch deception status", e);
    }
  };

  useEffect(() => {
    fetchStatus();
    const interval = setInterval(fetchStatus, 3000);
    return () => clearInterval(interval);
  }, []);

  const handleSimulate = async () => {
    try {
      const filesLost = Math.floor(Math.random() * 5); // 0 to 4 files lost
      const result = await invoke<string>("simulate_deception_trip", {
        args: {
          depth,
          density,
          files_lost: filesLost,
          tripped: true
        }
      });
      setFeed(prev => [`[Trip] Depth: ${depth}, Density: ${density} -> ${result} (Files lost: ${filesLost})`, ...prev].slice(0, 5));
      fetchStatus();
    } catch (e) {
      console.error(e);
      setFeed(prev => [`[Error] ${e}`, ...prev].slice(0, 5));
    }
  };

  return (
    <div style={{ background: "var(--panel)", border: "1.5px solid var(--hair)", borderRadius: 14, padding: 24, display: "flex", flexDirection: "column", gap: 24 }}>
      <div>
        <div style={{ fontSize: 20, fontWeight: 700, color: "var(--hi)", display: "flex", alignItems: "center", gap: 8 }}>
          <Crosshair size={24} color="#f43f5e" />
          FortiChain DeceptionNet™ (Q-Learning)
        </div>
        <div style={{ fontSize: 13, color: "var(--lo)", marginTop: 6, lineHeight: 1.5 }}>
          Adaptive decoy placement powered by Reinforcement Learning. State = (depth, density), Actions = [Sparse, DFS_First, Dense, EntropyHotspot].
          Convergence towards optimal placement via Bellman updates and epsilon-decay.
        </div>
      </div>

      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: 16 }}>
        <div className="card-punched" style={{ gap: 8 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 8, color: "var(--teal)" }}>
            <Brain size={16} /> <b>Exploration Rate (ε)</b>
          </div>
          <div style={{ fontSize: 24, fontWeight: 700, color: "var(--hi)" }}>
            {status ? status.epsilon.toFixed(4) : "..."}
          </div>
        </div>
        
        <div className="card-punched" style={{ gap: 8 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 8, color: "#a855f7" }}>
            <Activity size={16} /> <b>Q-Table Size</b>
          </div>
          <div style={{ fontSize: 24, fontWeight: 700, color: "var(--hi)" }}>
            {status ? status.q_table_size : "..."} States
          </div>
        </div>
        
        <div className="card-punched" style={{ gap: 8 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 8, color: "#ef4444" }}>
            <Shield size={16} /> <b>Total Episodes</b>
          </div>
          <div style={{ fontSize: 24, fontWeight: 700, color: "var(--hi)" }}>
            {status ? status.total_episodes : "..."}
          </div>
        </div>
      </div>

      <div style={{ border: "1px solid var(--line)", borderRadius: 8, padding: 16, background: "var(--void)" }}>
        <h4 style={{ color: "var(--hi)", marginTop: 0 }}>Simulate Trip Event</h4>
        <div style={{ display: "flex", gap: 16, alignItems: "center", marginTop: 12 }}>
          <label style={{ color: "var(--lo)", fontSize: 13 }}>
            Depth Bucket:
            <input type="number" min={0} max={10} value={depth} onChange={e => setDepth(Number(e.target.value))} style={{ marginLeft: 8, background: "var(--panel)", color: "var(--hi)", border: "1px solid var(--hair)", padding: "4px 8px", borderRadius: 4, width: 60 }} />
          </label>
          <label style={{ color: "var(--lo)", fontSize: 13 }}>
            Density Bucket:
            <input type="number" min={0} max={10} value={density} onChange={e => setDensity(Number(e.target.value))} style={{ marginLeft: 8, background: "var(--panel)", color: "var(--hi)", border: "1px solid var(--hair)", padding: "4px 8px", borderRadius: 4, width: 60 }} />
          </label>
          <button onClick={handleSimulate} style={{ padding: "6px 16px", background: "#f43f5e", color: "white", border: "none", borderRadius: 6, cursor: "pointer", fontWeight: 600 }}>
            Simulate RL Attack Vector
          </button>
        </div>
      </div>

      <div>
        <h4 style={{ color: "var(--hi)", marginBottom: 12 }}>Live Telemetry Feed</h4>
        <div style={{ background: "#0b132b", border: "1px solid var(--hair)", borderRadius: 8, padding: 16, fontFamily: "monospace", fontSize: 13, minHeight: 120, display: "flex", flexDirection: "column", gap: 6, color: "#10b981" }}>
          {feed.length === 0 ? <span style={{ color: "var(--lo)" }}>Waiting for events...</span> : null}
          {feed.map((line, i) => (
            <div key={i}>{line}</div>
          ))}
        </div>
      </div>
    </div>
  );
}
