const STORAGE_KEY = 'cm-guide-version';

export class GuideState {
    constructor(storage, version) {
        this.storage = storage;
        this.version = version;
    }

    shouldOpen() {
        try {
            return this.storage.getItem(STORAGE_KEY) !== this.version;
        } catch (_) {
            return true;
        }
    }

    complete() {
        try { this.storage.setItem(STORAGE_KEY, this.version); } catch (_) {}
    }

    reset() {
        try { this.storage.removeItem(STORAGE_KEY); } catch (_) {}
    }
}

export function bindGuide(root, state, translate) {
    const dialog = root.getElementById('app-guide');
    const title = root.getElementById('guide-step-title');
    const body = root.getElementById('guide-step-body');
    const visual = root.getElementById('guide-visual');
    const back = root.getElementById('guide-back');
    const next = root.getElementById('guide-next');
    const skip = root.getElementById('guide-skip');
    const reopen = root.getElementById('open-guide');
    const dots = [...root.querySelectorAll('.guide-dot')];
    const steps = [
        { title: 'guideStepLiveTitle', body: 'guideStepLiveBody', visual: 'live' },
        { title: 'guideStepWidgetTitle', body: 'guideStepWidgetBody', visual: 'widget' },
        { title: 'guideStepHistoryTitle', body: 'guideStepHistoryBody', visual: 'history' },
    ];
    let index = 0;

    const render = () => {
        const step = steps[index];
        title.textContent = translate(step.title);
        body.textContent = translate(step.body);
        visual.dataset.step = step.visual;
        back.disabled = index === 0;
        back.textContent = translate('guideBack');
        next.textContent = translate(index === steps.length - 1 ? 'guideDone' : 'guideNext');
        skip.textContent = translate('guideSkip');
        dots.forEach((dot, dotIndex) => dot.classList.toggle('active', dotIndex === index));
    };
    const open = (force = false) => {
        if (!force && !state.shouldOpen()) return;
        index = 0;
        render();
        dialog.classList.remove('hidden');
        dialog.setAttribute('aria-hidden', 'false');
        next.focus();
    };
    const close = (complete = true) => {
        if (complete) state.complete();
        dialog.classList.add('hidden');
        dialog.setAttribute('aria-hidden', 'true');
    };
    back.addEventListener('click', () => { if (index > 0) { index -= 1; render(); } });
    next.addEventListener('click', () => {
        if (index === steps.length - 1) close(true);
        else { index += 1; render(); }
    });
    skip.addEventListener('click', () => close(true));
    reopen.addEventListener('click', () => open(true));
    dialog.addEventListener('keydown', (event) => {
        if (event.key === 'Escape') close(true);
    });
    return { open, close, render };
}
