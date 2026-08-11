import test from 'node:test';
import assert from 'node:assert/strict';

import { buildDiagnosticModel } from '../src/diagnostics.js';

const translate = (key) => `<${key}>`;

const issues = [
    {
        key: 'wifi',
        severity: 'critical',
        penalty: 40,
        value: -82,
        unit: 'dBm',
        recommendation_key: 'recommendation_wifi',
    },
    {
        key: 'jitter',
        severity: 'poor',
        penalty: 30,
        value: 35.2,
        unit: 'ms',
        recommendation_key: 'recommendation_jitter',
    },
    {
        key: 'packet_loss',
        severity: 'fair',
        penalty: 10,
        value: 2,
        unit: '%',
        recommendation_key: 'recommendation_packet_loss',
    },
];

test('online diagnosis summarizes only the two largest measured penalties', () => {
    const model = buildDiagnosticModel(
        { connection_status: 'online', quality_issues: issues },
        translate,
    );

    assert.deepEqual(model.summary, [
        '<diagnostic_issue_wifi>: -82 dBm',
        '<diagnostic_issue_jitter>: 35.2 ms',
    ]);
    assert.equal(model.recommendation, '<recommendation_wifi>');
    assert.equal(model.rows.length, 3);
    assert.equal(model.rows[2].penalty, '-10');
});

test('connecting diagnosis explains that measurements are still being collected', () => {
    const model = buildDiagnosticModel(
        { connection_status: 'connecting', quality_issues: [] },
        translate,
    );
    assert.deepEqual(model.summary, ['<diagnostic_connecting_reason>']);
    assert.equal(model.recommendation, '<diagnostic_connecting_action>');
});

test('offline diagnosis provides recovery guidance', () => {
    const model = buildDiagnosticModel(
        { connection_status: 'offline', quality_issues: [] },
        translate,
    );
    assert.deepEqual(model.summary, ['<diagnostic_offline_reason>']);
    assert.equal(model.recommendation, '<diagnostic_offline_action>');
});

test('healthy diagnosis confirms measured values are within limits', () => {
    const model = buildDiagnosticModel(
        { connection_status: 'online', quality_issues: [] },
        translate,
    );
    assert.deepEqual(model.summary, ['<diagnostic_healthy_reason>']);
    assert.equal(model.recommendation, '<diagnostic_healthy_action>');
    assert.deepEqual(model.rows, []);
});
