import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState, useCallback } from "react";
import type { OverviewStats, Incident, IncidentDetail } from "../lib/forensics-types";

export function useOverviewStats() {
  const [stats, setStats] = useState<OverviewStats | null>(null);

  const refresh = useCallback(async () => {
    try {
      const res = await invoke<OverviewStats>("get_overview_stats");
      setStats(res);
    } catch (e) {
      console.error("Failed to fetch overview stats:", e);
    }
  }, []);

  useEffect(() => {
    refresh();
    const unlisten = listen("forensics://new-event", () => refresh());
    return () => {
      unlisten.then((f) => f());
    };
  }, [refresh]);

  return { stats, refresh };
}

export function useIncidents(page: number, pageSize: number, filters: { severity?: string; status?: string }) {
  const [incidents, setIncidents] = useState<Incident[]>([]);
  const [total, setTotal] = useState<number>(0);
  const [loading, setLoading] = useState(true);

  const fetchIncidents = useCallback(async () => {
    setLoading(true);
    try {
      const res = await invoke<{ rows: Incident[]; total: number }>("list_incidents", {
        page,
        pageSize,
        severity: filters.severity || null,
        status: filters.status || null,
      });
      setIncidents(res.rows);
      setTotal(res.total);
    } catch (e) {
      console.error("Failed to fetch incidents:", e);
    } finally {
      setLoading(false);
    }
  }, [page, pageSize, filters.severity, filters.status]);

  useEffect(() => {
    fetchIncidents();
    const unlisten = listen("forensics://new-event", () => fetchIncidents());
    return () => {
      unlisten.then((f) => f());
    };
  }, [fetchIncidents]);

  return { incidents, total, loading, refetch: fetchIncidents };
}

export function useIncidentDetail(incidentId: string | null) {
  const [detail, setDetail] = useState<IncidentDetail | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!incidentId) {
      setDetail(null);
      return;
    }
    setLoading(true);
    invoke<IncidentDetail>("get_incident_detail", { incidentId })
      .then(setDetail)
      .catch((err) => console.error(err))
      .finally(() => setLoading(false));
  }, [incidentId]);

  return { detail, loading };
}
