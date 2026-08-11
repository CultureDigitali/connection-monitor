import test from 'node:test';
import assert from 'node:assert/strict';

import { buildFloatingModel } from '../src/floating-model.js';

const translate = (key) => key;

test('online widget formats four independent live indicators', () => {
    const model = buildFloatingModel({
        download_mbps: 1.5,
        upload_mbps: 0.25,
        total_download_mb: 10,
        total_upload_mb: 2,
        quality_score: 84,
        connection_status: 'online',
    }, translate);

    assert.deepEqual(model.download, {
        value: '1.5', unit: 'Mbps', state: 'download',
        label: 'floatingDownload', tooltip: 'tooltipWidgetDownload',
    });
    assert.equal(model.upload.value, '250');
    assert.equal(model.upload.unit, 'Kbps');
    assert.equal(model.quality.value, '84');
    assert.equal(model.quality.unit, '/100');
    assert.equal(model.quality.state, 'good');
    assert.equal(model.data.value, '12 MB');
    assert.equal(model.data.tooltip, 'tooltipWidgetData');
});

test('connecting and offline states never present a false quality score', () => {
    const connecting = buildFloatingModel({ connection_status: 'connecting' }, translate);
    const offline = buildFloatingModel({ connection_status: 'offline' }, translate);

    assert.equal(connecting.quality.value, '…');
    assert.equal(connecting.quality.state, 'connecting');
    assert.equal(connecting.quality.label, 'statusConnecting');
    assert.equal(offline.quality.value, '—');
    assert.equal(offline.quality.state, 'offline');
    assert.equal(offline.quality.label, 'statusDisconnected');
});

test('quality state follows the established score thresholds', () => {
    const state = (score) => buildFloatingModel({
        quality_score: score,
        connection_status: 'online',
    }, translate).quality.state;

    assert.equal(state(90), 'excellent');
    assert.equal(state(75), 'good');
    assert.equal(state(50), 'fair');
    assert.equal(state(25), 'poor');
    assert.equal(state(24), 'critical');
});
