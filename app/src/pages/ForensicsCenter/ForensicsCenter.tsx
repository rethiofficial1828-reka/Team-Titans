import { useState } from "react";
import { useOverviewStats, useIncidents, useIncidentDetail } from "../../hooks/useForensics";

export default function ForensicsCenter() {
  const { stats } = useOverviewStats();
  const [page, setPage] = useState(1);
  const pageSize = 10;
  const [severityFilter, setSeverityFilter] = useState<string>("");
  const [statusFilter, setStatusFilter] = useState<string>("");
  const [selectedIncidentId, setSelectedIncidentId] = useState<string | null>(null);

  const { incidents, total, loading } = useIncidents(page, pageSize, {
    severity: severityFilter,
    status: statusFilter,
  });

  const { detail, loading: detailLoading } = useIncidentDetail(selectedIncidentId);

  const getSeverityStyle = (sev: string) => {
    switch (sev) {
      case "CRITICAL":
        return "stamp stamp-critical";
      case "HIGH":
        return "stamp stamp-exposed";
      case "MEDIUM":
        return "stamp stamp-exposed";
      default:
        return "stamp stamp-dim";
    }
  };

  return (
    <div className="p-6 space-y-6 text-gray-100 font-sans">
      {/* Header */}
      <div className="flex justify-between items-center border-b border-[var(--line)] pb-4">
        <div>
          <h1 className="text-2xl font-bold text-[var(--teal)] tracking-wide flex items-center gap-2">
            <span>🛡️</span> Attack Forensics & Threat Intelligence Center
          </h1>
          <p className="text-xs text-[var(--lo)] mt-1">
            Real-time attack classification, SHA3-512 tamper-proof hash ledger & incident correlation engine
          </p>
        </div>
        <span className="stamp stamp-accent animate-pulse">
          ● LIVE EVENT LISTENER ACTIVE
        </span>
      </div>

      {/* Stats Cards Grid */}
      <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
        <div className="card-punched">
          <div className="section-header">Total Incidents</div>
          <div className="text-2xl font-mono font-bold text-[var(--hi)] mt-2">{stats?.totalIncidents ?? 0}</div>
        </div>
        <div className="card-punched">
          <div className="section-header" style={{ color: "var(--red)" }}>Critical Threats</div>
          <div className="text-2xl font-mono font-bold text-[var(--red)] mt-2">{stats?.critical ?? 0}</div>
        </div>
        <div className="card-punched">
          <div className="section-header" style={{ color: "var(--green)" }}>Blocked Threats</div>
          <div className="text-2xl font-mono font-bold text-[var(--green)] mt-2">{stats?.blocked ?? 0}</div>
        </div>
        <div className="card-punched">
          <div className="section-header" style={{ color: "var(--amber)" }}>Avg Risk Score</div>
          <div className="text-2xl font-mono font-bold text-[var(--amber)] mt-2">{stats?.avgRisk ?? 0}/100</div>
        </div>
        <div className="card-punched">
          <div className="section-header" style={{ color: "var(--teal)" }}>Protected Vaults</div>
          <div className="text-2xl font-mono font-bold text-[var(--teal)] mt-2">{stats?.protectedFolders ?? 0}</div>
        </div>
      </div>

      {/* Filter Bar */}
      <div className="flex gap-4 bg-[var(--panel-2)] border border-[var(--line)] p-4 rounded-xl shadow-lg">
        <div className="flex-1">
          <label className="text-xs text-[var(--lo)] font-medium block mb-1">Filter Severity</label>
          <select
            value={severityFilter}
            onChange={(e) => {
              setSeverityFilter(e.target.value);
              setPage(1);
            }}
            className="w-full bg-[var(--void)] border border-[var(--line)] rounded-lg px-3 py-2 text-sm text-[var(--hi)] focus:border-[var(--teal)] focus:outline-none transition-colors"
          >
            <option value="">All Severities</option>
            <option value="CRITICAL">CRITICAL</option>
            <option value="HIGH">HIGH</option>
            <option value="MEDIUM">MEDIUM</option>
            <option value="LOW">LOW</option>
          </select>
        </div>
        <div className="flex-1">
          <label className="text-xs text-[var(--lo)] font-medium block mb-1">Filter Status</label>
          <select
            value={statusFilter}
            onChange={(e) => {
              setStatusFilter(e.target.value);
              setPage(1);
            }}
            className="w-full bg-[var(--void)] border border-[var(--line)] rounded-lg px-3 py-2 text-sm text-[var(--hi)] focus:border-[var(--teal)] focus:outline-none transition-colors"
          >
            <option value="">All Statuses</option>
            <option value="OPEN">OPEN</option>
            <option value="BLOCKED">BLOCKED</option>
            <option value="RESOLVED">RESOLVED</option>
            <option value="ALLOWED">ALLOWED</option>
          </select>
        </div>
      </div>

      {/* Incidents Table */}
      <div className="bg-[var(--panel)] border border-[var(--line)] rounded-xl overflow-hidden shadow-lg shadow-black/50">
        <table className="w-full text-left text-sm">
          <thead className="bg-[var(--void)] text-xs text-[var(--lo)] uppercase border-b border-[var(--line)] font-semibold tracking-wider">
            <tr>
              <th className="p-4">Incident ID</th>
              <th className="p-4">Attack Type</th>
              <th className="p-4">Severity</th>
              <th className="p-4">Risk Score</th>
              <th className="p-4">Target Folder</th>
              <th className="p-4">Status</th>
              <th className="p-4 text-right">Action</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-[var(--line)]/60">
            {loading ? (
              <tr>
                <td colSpan={7} className="p-8 text-center text-[var(--dim)] animate-pulse">
                  Loading attack forensics ledger...
                </td>
              </tr>
            ) : incidents.length === 0 ? (
              <tr>
                <td colSpan={7} className="p-8 text-center text-[var(--dim)] font-medium">
                  No attack incidents recorded yet. Trigger a folder protection event to test.
                </td>
              </tr>
            ) : (
              incidents.map((inc) => (
                <tr key={inc.incident_id} className="hover:bg-[var(--panel-2)] transition-colors">
                  <td className="p-4 font-mono font-bold text-[var(--teal)]">{inc.incident_id}</td>
                  <td className="p-4 text-[var(--hi)]">{inc.attack_type}</td>
                  <td className="p-4">
                    <span className={getSeverityStyle(inc.severity)}>
                      {inc.severity}
                    </span>
                  </td>
                  <td className="p-4 font-mono font-medium text-[var(--hi)]">{inc.risk_score}/100</td>
                  <td className="p-4 font-mono text-xs text-[var(--lo)]">{inc.target_folder || "—"}</td>
                  <td className="p-4">
                    <span className="stamp stamp-dim">
                      {inc.status}
                    </span>
                  </td>
                  <td className="p-4 text-right">
                    <button
                      onClick={() => setSelectedIncidentId(inc.incident_id)}
                      className="btn-ghost"
                    >
                      Inspect Chain
                    </button>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>

        {/* Pagination */}
        <div className="flex justify-between items-center p-4 bg-[var(--void2)] border-t border-[var(--line)] text-xs text-[var(--lo)] font-medium">
          <span>Showing page {page} of {Math.ceil(total / pageSize) || 1} ({total} total incidents)</span>
          <div className="flex gap-2">
            <button
              disabled={page <= 1}
              onClick={() => setPage((p) => Math.max(1, p - 1))}
              className="px-3 py-1 bg-[var(--panel-2)] hover:bg-[var(--panel-3)] text-[var(--hi)] disabled:opacity-40 rounded border border-[var(--line)] transition-colors"
            >
              Previous
            </button>
            <button
              disabled={page * pageSize >= total}
              onClick={() => setPage((p) => p + 1)}
              className="px-3 py-1 bg-[var(--panel-2)] hover:bg-[var(--panel-3)] text-[var(--hi)] disabled:opacity-40 rounded border border-[var(--line)] transition-colors"
            >
              Next
            </button>
          </div>
        </div>
      </div>

      {/* Incident Detail Modal */}
      {selectedIncidentId && (
        <div className="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-6">
          <div className="bg-[var(--void)] border border-[var(--line)] rounded-2xl max-w-4xl w-full max-h-[90vh] overflow-y-auto p-6 space-y-6 shadow-2xl">
            <div className="flex justify-between items-start border-b border-[var(--line)] pb-4">
              <div>
                <h2 className="text-xl font-bold text-[var(--hi)] flex items-center gap-2">
                  Incident Forensics: <span className="font-mono text-[var(--teal)]">{selectedIncidentId}</span>
                </h2>
                <p className="text-xs text-[var(--lo)] mt-1">Cryptographically chained SHA3-512 forensic record & timeline</p>
              </div>
              <button
                onClick={() => setSelectedIncidentId(null)}
                className="text-[var(--lo)] hover:text-[var(--hi)] px-3 py-1 rounded bg-[var(--panel-2)] transition-colors"
              >
                ✕ Close
              </button>
            </div>

            {detailLoading || !detail ? (
              <div className="p-12 text-center text-[var(--dim)] animate-pulse">Loading incident evidence chain...</div>
            ) : (
              <div className="space-y-6 text-sm">
                {/* Timeline */}
                <div>
                  <h3 className="section-header mb-3">Attack Timeline Steps</h3>
                  <div className="space-y-2">
                    {detail.timeline.map((st) => (
                      <div key={st.step_order} className="flex items-center gap-4 bg-[var(--panel)] p-3 rounded-lg border border-[var(--line)]">
                        <span className="font-mono text-xs text-[var(--teal)] font-bold">#{st.step_order}</span>
                        <span className="font-semibold text-[var(--hi)]">{st.label}</span>
                        <span className="text-xs text-[var(--dim)] font-mono">
                          {new Date(st.timestamp).toLocaleTimeString()}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>

                {/* Recommendations */}
                <div>
                  <h3 className="section-header mb-3">Recommended Security Actions</h3>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                    {detail.recommendations.map((rec, i) => (
                      <div key={i} className="bg-[var(--panel)] p-3 rounded-lg border border-[var(--teal)]/40 flex justify-between items-center shadow-md">
                        <span className="text-[var(--teal)] text-xs font-medium">⚡ {rec.recommendation}</span>
                        <span className="stamp stamp-exposed">
                          {rec.priority}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>

                {/* SHA3-512 Hash Chain Evidence */}
                <div>
                  <h3 className="section-header mb-3">
                    🔐 SHA3-512 Cryptographic Hash Chain Ledger
                  </h3>
                  <div className="space-y-3 font-mono text-xs">
                    {detail.logs.map((log) => (
                      <div key={log.log_id} className="bg-[var(--panel)] p-4 rounded-xl border border-[var(--line)] space-y-2 shadow-inner">
                        <div className="flex justify-between text-[var(--lo)] border-b border-[var(--line)] pb-2">
                          <span>Action: {log.action_taken}</span>
                          <span>{new Date(log.timestamp).toLocaleString()}</span>
                        </div>
                        <div className="text-[var(--hi)]">
                          Target: <span className="text-white">{log.target_folder}</span>
                        </div>
                        <div className="text-[var(--green)] break-all font-bold">
                          SHA3 Hash: {log.sha3_hash}
                        </div>
                        <div className="text-[var(--dim)] break-all">
                          Prev Hash: {log.prev_hash}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
