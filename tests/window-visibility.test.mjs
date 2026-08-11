import test from 'node:test';
import assert from 'node:assert/strict';

import { bindWindowVisibility } from '../src/window-visibility.js';

test('ordinary document clicks do not hide the widget', () => {
    const closeButton = new EventTarget();
    const documentTarget = new EventTarget();
    let hideCount = 0;

    bindWindowVisibility(closeButton, () => { hideCount += 1; });
    documentTarget.dispatchEvent(new Event('click'));

    assert.equal(hideCount, 0);
});

test('the explicit close button hides the widget once', () => {
    const closeButton = new EventTarget();
    let hideCount = 0;

    bindWindowVisibility(closeButton, () => { hideCount += 1; });
    closeButton.dispatchEvent(new Event('click'));

    assert.equal(hideCount, 1);
});
