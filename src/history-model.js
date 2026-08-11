import { formatData } from './formatters.js';

function finiteOrNull(value) {
    return Number.isFinite(value) ? value : null;
}

function formatQuality(value) {
    return Number.isFinite(value) ? `${Math.round(value)}/100` : '—';
}

function formatAvailability(value, hasData) {
    return hasData && Number.isFinite(value) ? `${(value * 100).toFixed(1)}%` : '—';
}

export function selectReplayPoint(buckets = [], timestamp) {
    if (!buckets.length || !Number.isFinite(timestamp)) return null;
    return buckets.reduce((nearest, bucket) => (
        Math.abs(bucket.started_at - timestamp) < Math.abs(nearest.started_at - timestamp)
            ? bucket
            : nearest
    ));
}

export function findIncidentAt(incidents = [], timestamp) {
    return incidents.find((incident) => (
        incident.started_at <= timestamp
        && (incident.ended_at == null || incident.ended_at >= timestamp)
    )) || null;
}

function buildIncident(incident, translate) {
    const endedAt = Number.isFinite(incident.ended_at) ? incident.ended_at : null;
    return {
        ...incident,
        title: translate(incident.kind === 'offline' ? 'guardianOffline' : 'guardianDegraded'),
        cause: translate(`diagnostic_issue_${incident.issue_key || 'offline'}`),
        recommendation: translate(incident.recommendation_key || 'recommendation_offline'),
        measurement: Number.isFinite(incident.value)
            ? `${incident.value.toFixed(incident.unit === '%' ? 1 : 0)} ${incident.unit || ''}`.trim()
            : '',
        durationSeconds: endedAt == null ? null : Math.max(0, endedAt - incident.started_at),
    };
}

export function buildHistoryView(response = {}, translate = (key) => key) {
    const buckets = Array.isArray(response.buckets) ? response.buckets : [];
    const incidents = Array.isArray(response.incidents) ? response.incidents : [];
    const summary = response.summary || {};
    const streak = response.streak || {};
    const hasData = buckets.length > 0;
    const current = Math.max(0, Number(streak.current) || 0);
    const nextMilestone = Number(streak.next_milestone) || null;

    return {
        range: response.range || '24h',
        empty: !hasData,
        summary: {
            quality: formatQuality(summary.average_quality),
            minimumQuality: formatQuality(summary.minimum_quality),
            availability: formatAvailability(summary.availability, hasData),
            data: hasData
                ? formatData((Number(summary.downloaded_mb) || 0) + (Number(summary.uploaded_mb) || 0))
                : '—',
            incidents: String(Number(summary.incident_count) || 0),
        },
        points: buckets.map((bucket) => ({
            timestamp: bucket.started_at,
            quality: finiteOrNull(bucket.average_quality),
            download: finiteOrNull(bucket.average_download_mbps),
            upload: finiteOrNull(bucket.average_upload_mbps),
            ping: finiteOrNull(bucket.average_ping_ms),
            availability: finiteOrNull(bucket.availability),
            raw: bucket,
        })),
        incidents: incidents.map((incident) => buildIncident(incident, translate)),
        streak: {
            current,
            best: Math.max(0, Number(streak.best) || 0),
            nextMilestone,
            progress: nextMilestone ? Math.min(1, current / nextMilestone) : 1,
            todayReliable: streak.today_reliable_so_far ?? null,
        },
    };
}
