import test from 'node:test';
import assert from 'node:assert/strict';

import { applyOfficialLogo } from '../src/branding.js';

test('official logo is assigned to every branded image', () => {
    const images = [{ src: '', alt: '' }, { src: '', alt: '' }];
    applyOfficialLogo(images, '/assets/official.png', 'Connection Monitor');
    assert.deepEqual(images, [
        { src: '/assets/official.png', alt: 'Connection Monitor' },
        { src: '/assets/official.png', alt: 'Connection Monitor' },
    ]);
});
