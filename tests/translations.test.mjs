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
        'infoWebsite',
    ];
    for (const language of ['en', 'it', 'es', 'fr']) {
        for (const key of required) assert.ok(translations[language][key], `${language}.${key}`);
    }
});

test('every language has identical keys for statistics, Guardian, guide, and tooltips', () => {
    const required = [
        'tabStatistics', 'historyAverageQuality', 'historyAvailability', 'historyIncidents',
        'historyTransferred', 'historyEmpty', 'historyUnavailable', 'guardianTitle',
        'guardianDegraded', 'guardianOffline', 'guardianNoIncidents', 'streakReliableDays',
        'streakBest', 'guideTitle', 'guideStepLiveTitle', 'guideStepWidgetTitle',
        'guideStepHistoryTitle', 'guideNext', 'guideBack', 'guideDone', 'guideSkip',
        'tooltipWidgetDownload', 'tooltipWidgetUpload', 'tooltipWidgetQuality',
        'tooltipWidgetData', 'tooltipReplay', 'tooltipStreak',
    ];
    const languages = ['en', 'it', 'es', 'fr'];
    const englishKeys = Object.keys(translations.en).sort();
    for (const language of languages) {
        assert.deepEqual(Object.keys(translations[language]).sort(), englishKeys, `${language} key parity`);
        for (const key of required) assert.ok(translations[language][key], `${language}.${key}`);
    }
});

test('every language explains the Windows hidden tray menu', () => {
    for (const language of Object.values(translations)) {
        assert.match(language.guideStepWidgetBody, /Windows/);
        assert.match(language.guideStepWidgetBody, /\^/);
    }
});
