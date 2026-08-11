import { buildFloatingModel } from './floating-model.js';
import { i18n } from './i18n.js';
import { bindTooltips } from './tooltips.js';

let stats = {};
const cleanups = [];
const cleanupTooltips = bindTooltips(document, (key) => i18n.t(key));

function render() {
    const model = buildFloatingModel(stats, (key) => i18n.t(key));
    for (const [key, value] of Object.entries(model)) {
        const element = document.querySelector(`[data-metric="${key}"]`);
        if (!element) continue;
        element.className = `floating-metric ${key} ${value.state}`;
        element.querySelector('.metric-number').textContent = value.value;
        element.querySelector('.metric-unit').textContent = value.unit;
        element.querySelector('.metric-name').textContent = value.label;
        element.title = value.tooltip;
        element.setAttribute('aria-label', `${value.label}: ${value.value} ${value.unit}`.trim());
    }
}

async function initialize() {
    if (!window.__TAURI_INTERNALS__) {
        render();
        return;
    }
    const { invoke } = window.__TAURI__.core;
    const { listen } = window.__TAURI__.event;
    try {
        i18n.setLanguage(await invoke('get_language'));
        stats = await invoke('get_bandwidth');
        render();
    } catch (error) {
        console.error('Failed to initialize floating widget:', error);
    }

    cleanups.push(await listen('stats-update', (event) => {
        stats = event.payload;
        render();
    }));
    cleanups.push(await listen('language-changed', (event) => {
        i18n.setLanguage(event.payload);
        render();
    }));
}

document.getElementById('floating-close').addEventListener('click', () => {
    if (window.__TAURI_INTERNALS__) {
        window.__TAURI__.core.invoke('toggle_floating_window').catch(() => {});
    }
});

window.addEventListener('beforeunload', () => {
    for (const cleanup of cleanups) cleanup();
    cleanupTooltips();
});

initialize();
