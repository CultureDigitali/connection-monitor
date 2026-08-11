import test from 'node:test';
import assert from 'node:assert/strict';

import { bindTooltips } from '../src/tooltips.js';

class FakeElement extends EventTarget {
    constructor(key) {
        super();
        this.dataset = { tooltipKey: key };
        this.attributes = new Map();
    }
    setAttribute(name, value) { this.attributes.set(name, String(value)); }
    removeAttribute(name) { this.attributes.delete(name); }
    getBoundingClientRect() { return { left: 20, top: 20, right: 60, bottom: 40, width: 40, height: 20 }; }
}

function surface() {
    const classes = new Set(['hidden']);
    return {
        id: 'app-tooltip', textContent: '', style: {},
        classList: {
            add: (name) => classes.add(name),
            remove: (name) => classes.delete(name),
            contains: (name) => classes.has(name),
        },
    };
}

test('keyboard focus shows translated tooltip and Escape hides it', () => {
    const element = new FakeElement('tooltipQuality');
    const tooltip = surface();
    const root = { querySelectorAll: () => [element], ownerDocument: { defaultView: { innerWidth: 300, innerHeight: 200 } } };
    const cleanup = bindTooltips(root, (key) => `translated:${key}`, { surface: tooltip });

    element.dispatchEvent(new Event('focusin'));
    assert.equal(tooltip.textContent, 'translated:tooltipQuality');
    assert.equal(tooltip.classList.contains('hidden'), false);
    assert.equal(element.attributes.get('aria-describedby'), 'app-tooltip');
    element.dispatchEvent(new Event('keydown', { bubbles: true }));
    const escape = new Event('keydown');
    Object.defineProperty(escape, 'key', { value: 'Escape' });
    element.dispatchEvent(escape);
    assert.equal(tooltip.classList.contains('hidden'), true);

    cleanup();
    element.dispatchEvent(new Event('focusin'));
    assert.equal(tooltip.classList.contains('hidden'), true);
});
