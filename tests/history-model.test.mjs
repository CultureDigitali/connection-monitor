import test from 'node:test';
import assert from 'node:assert/strict';

import { buildHistoryView, findIncidentAt, selectReplayPoint } from '../src/history-model.js';

const translate = (key) => key;

test('empty history stays explicit instead of inventing zero measurements', () => {
    const model = buildHistoryView({
        range: '24h',
        summary: {
            average_quality: null,
            minimum_quality: null,
            availability: 0,
            downloaded_mb: 0,
            uploaded_mb: 0,
            incident_count: 0,
        },
        buckets: [], incidents: [],
        streak: { current: 0, best: 0, next_milestone: 3, today_reliable_so_far: null },
    }, translate);

    assert.equal(model.empty, true);
    assert.equal(model.summary.quality, '—');
    assert.equal(model.summary.availability, '—');
    assert.deepEqual(model.points, []);
});

test('history view formats summaries and preserves missing chart points', () => {
    const model = buildHistoryView({
        range: '7d',
        summary: {
            average_quality: 82.4,
            minimum_quality: 35,
            availability: 0.995,
            downloaded_mb: 1500,
            uploaded_mb: 500,
            incident_count: 2,
        },
        buckets: [
            { started_at: 100, average_quality: 80, average_download_mbps: 4, average_upload_mbps: 1, average_ping_ms: 20 },
            { started_at: 160, average_quality: null, average_download_mbps: null, average_upload_mbps: null, average_ping_ms: null },
        ],
        incidents: [],
        streak: { current: 2, best: 7, next_milestone: 3, today_reliable_so_far: true },
    }, translate);

    assert.equal(model.empty, false);
    assert.equal(model.summary.quality, '82/100');
    assert.equal(model.summary.availability, '99.5%');
    assert.equal(model.summary.data, '2.00 GB');
    assert.equal(model.summary.incidents, '2');
    assert.equal(model.points[1].quality, null);
    assert.equal(model.streak.progress, 2 / 3);
});

test('Replay selects the nearest real bucket and active incident', () => {
    const buckets = [{ started_at: 100 }, { started_at: 160 }];
    assert.equal(selectReplayPoint(buckets, 145).started_at, 160);
    assert.equal(selectReplayPoint([], 145), null);

    const incidents = [
        { id: 'a', started_at: 110, ended_at: 130 },
        { id: 'b', started_at: 140, ended_at: null },
    ];
    assert.equal(findIncidentAt(incidents, 120).id, 'a');
    assert.equal(findIncidentAt(incidents, 150).id, 'b');
    assert.equal(findIncidentAt(incidents, 100), null);
});

test('Guardian incident model exposes measured cause and recommendation', () => {
    const model = buildHistoryView({
        range: '24h',
        summary: { average_quality: 40, minimum_quality: 10, availability: 0.9, downloaded_mb: 0, uploaded_mb: 0, incident_count: 1 },
        buckets: [{ started_at: 100, average_quality: 40 }],
        incidents: [{
            id: 'incident-1', kind: 'degraded', started_at: 100, ended_at: 160,
            issue_key: 'latency', value: 210, unit: 'ms', recommendation_key: 'recommendation_latency',
        }],
        streak: { current: 0, best: 0, next_milestone: 3, today_reliable_so_far: false },
    }, translate);

    assert.equal(model.incidents[0].title, 'guardianDegraded');
    assert.equal(model.incidents[0].cause, 'diagnostic_issue_latency');
    assert.equal(model.incidents[0].recommendation, 'recommendation_latency');
    assert.equal(model.incidents[0].durationSeconds, 60);
});
