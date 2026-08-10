import test from 'node:test';
import assert from 'node:assert/strict';

import { formatData, formatRate } from '../src/formatters.js';

test('formatRate selects Kbps, Mbps, and Gbps', () => {
    assert.deepEqual(formatRate(0.2), { value: '200', unit: 'Kbps' });
    assert.deepEqual(formatRate(1.7), { value: '1.7', unit: 'Mbps' });
    assert.deepEqual(formatRate(1200), { value: '1.20', unit: 'Gbps' });
});

test('formatData selects MB, GB, and TB', () => {
    assert.equal(formatData(500), '500 MB');
    assert.equal(formatData(1500), '1.50 GB');
    assert.equal(formatData(1_500_000), '1.50 TB');
});
