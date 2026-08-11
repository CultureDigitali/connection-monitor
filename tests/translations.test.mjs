import test from 'node:test';
import assert from 'node:assert/strict';

import { translations } from '../src/i18n.js';

test('every supported language calls the redesigned section Info', () => {
    for (const language of ['en', 'it', 'es', 'fr']) {
        assert.equal(translations[language].tabInfo, 'Info');
    }
});

test('every supported language includes diagnostic labels and recommendations', () => {
    const required = [
        'diagnosticWhy',
        'diagnosticAction',
        'diagnostic_issue_latency',
        'diagnostic_issue_jitter',
        'diagnostic_issue_packet_loss',
        'diagnostic_issue_wifi',
        'recommendation_latency',
        'recommendation_jitter',
        'recommendation_packet_loss',
        'recommendation_wifi',
    ];
    for (const language of ['en', 'it', 'es', 'fr']) {
        for (const key of required) assert.ok(translations[language][key], `${language}.${key}`);
    }
});
