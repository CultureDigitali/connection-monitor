import test from 'node:test';
import assert from 'node:assert/strict';

import { GuideState } from '../src/guide.js';

class MapStorage {
    constructor() { this.values = new Map(); }
    getItem(key) { return this.values.get(key) ?? null; }
    setItem(key, value) { this.values.set(key, String(value)); }
    removeItem(key) { this.values.delete(key); }
}

test('guide opens once per guide version and can be reopened explicitly', () => {
    const storage = new MapStorage();
    const guide = new GuideState(storage, '0.3');
    assert.equal(guide.shouldOpen(), true);
    guide.complete();
    assert.equal(new GuideState(storage, '0.3').shouldOpen(), false);
    assert.equal(new GuideState(storage, '0.4').shouldOpen(), true);
    guide.reset();
    assert.equal(guide.shouldOpen(), true);
});

test('storage denial keeps guide usable without crashing', () => {
    const denied = {
        getItem() { throw new Error('denied'); },
        setItem() { throw new Error('denied'); },
        removeItem() { throw new Error('denied'); },
    };
    const guide = new GuideState(denied, '0.3');
    assert.equal(guide.shouldOpen(), true);
    assert.doesNotThrow(() => guide.complete());
    assert.doesNotThrow(() => guide.reset());
});
