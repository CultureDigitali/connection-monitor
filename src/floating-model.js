import { formatData, formatRate } from './formatters.js';

function qualityState(score) {
    if (score >= 90) return 'excellent';
    if (score >= 75) return 'good';
    if (score >= 50) return 'fair';
    if (score >= 25) return 'poor';
    return 'critical';
}

function qualityLabel(score, translate) {
    if (score >= 90) return translate('qualityExcellent');
    if (score >= 75) return translate('qualityGood');
    if (score >= 50) return translate('qualityFair');
    if (score >= 25) return translate('qualityPoor');
    return translate('qualityCritical');
}

export function buildFloatingModel(stats = {}, translate = (key) => key) {
    const download = formatRate(stats.download_mbps);
    const upload = formatRate(stats.upload_mbps);
    const status = stats.connection_status || 'connecting';
    let quality;
    if (status === 'online') {
        const score = Math.max(0, Math.min(100, Number(stats.quality_score) || 0));
        quality = {
            value: String(Math.round(score)),
            unit: '/100',
            state: qualityState(score),
            label: qualityLabel(score, translate),
            tooltip: translate('tooltipWidgetQuality'),
        };
    } else {
        const offline = status === 'offline';
        quality = {
            value: offline ? '—' : '…',
            unit: '',
            state: offline ? 'offline' : 'connecting',
            label: translate(offline ? 'statusDisconnected' : 'statusConnecting'),
            tooltip: translate('tooltipWidgetQuality'),
        };
    }

    return {
        download: {
            ...download,
            state: 'download',
            label: translate('floatingDownload'),
            tooltip: translate('tooltipWidgetDownload'),
        },
        upload: {
            ...upload,
            state: 'upload',
            label: translate('floatingUpload'),
            tooltip: translate('tooltipWidgetUpload'),
        },
        quality,
        data: {
            value: formatData((Number(stats.total_download_mb) || 0) + (Number(stats.total_upload_mb) || 0)),
            unit: '',
            state: 'data',
            label: translate('floatingData'),
            tooltip: translate('tooltipWidgetData'),
        },
    };
}
