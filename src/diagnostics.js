function formatNumber(value, unit) {
    const number = Number(value) || 0;
    if (unit === 'dBm' || Number.isInteger(number)) return number.toFixed(0);
    return number.toFixed(1);
}

function formatIssue(issue, translate) {
    const label = translate(`diagnostic_issue_${issue.key}`);
    const measured = `${formatNumber(issue.value, issue.unit)} ${issue.unit}`;
    return {
        key: issue.key,
        severity: issue.severity,
        label,
        measured,
        summary: `${label}: ${measured}`,
        penalty: `-${issue.penalty}`,
    };
}

export function buildDiagnosticModel(stats, translate) {
    const state = stats.connection_status || 'connecting';
    if (state === 'connecting') {
        return {
            state,
            summary: [translate('diagnostic_connecting_reason')],
            recommendation: translate('diagnostic_connecting_action'),
            rows: [],
        };
    }
    if (state === 'offline') {
        return {
            state,
            summary: [translate('diagnostic_offline_reason')],
            recommendation: translate('diagnostic_offline_action'),
            rows: [],
        };
    }

    const issues = Array.isArray(stats.quality_issues) ? stats.quality_issues : [];
    if (issues.length === 0) {
        return {
            state: 'healthy',
            summary: [translate('diagnostic_healthy_reason')],
            recommendation: translate('diagnostic_healthy_action'),
            rows: [],
        };
    }

    const rows = issues.map((issue) => formatIssue(issue, translate));
    return {
        state,
        summary: rows.slice(0, 2).map((row) => row.summary),
        recommendation: translate(issues[0].recommendation_key),
        rows,
    };
}
