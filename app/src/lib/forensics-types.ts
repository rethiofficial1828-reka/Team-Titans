export interface OverviewStats {
  totalIncidents: number;
  critical: number;
  blocked: number;
  avgRisk: number;
  protectedFolders: number;
}

export interface Incident {
  incident_id: string;
  created_at: number;
  updated_at?: number;
  attack_type: string;
  severity: "LOW" | "MEDIUM" | "HIGH" | "CRITICAL";
  risk_score: number;
  status: "OPEN" | "BLOCKED" | "RESOLVED" | "ALLOWED";
  username?: string;
  computer_name?: string;
  target_folder?: string;
  event_count: number;
}

export interface AttackLogRecord {
  log_id?: number;
  incident_id: string;
  timestamp: number;
  attack_type: string;
  severity: string;
  risk_score: number;
  username?: string;
  computer_name?: string;
  process_name?: string;
  process_id?: number;
  executable_path?: string;
  target_folder?: string;
  target_file?: string;
  action_taken?: string;
  status: string;
  sha3_hash: string;
  prev_hash: string;
  remarks?: string;
}

export interface TimelineStep {
  step_order: number;
  label: string;
  timestamp: number;
  detail?: string;
}

export interface Recommendation {
  recommendation: string;
  priority: string;
  applied: boolean;
}

export interface IncidentDetail {
  incident: Incident;
  logs: AttackLogRecord[];
  timeline: TimelineStep[];
  recommendations: Recommendation[];
}
