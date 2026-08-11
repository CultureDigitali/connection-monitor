export function bindTooltips(root, translate, options = {}) {
    const doc = root.ownerDocument || root;
    const view = doc.defaultView || globalThis;
    const surface = options.surface || (() => {
        const element = doc.createElement('div');
        element.id = 'app-tooltip';
        element.className = 'app-tooltip hidden';
        element.setAttribute('role', 'tooltip');
        doc.body.appendChild(element);
        return element;
    })();
    const setTimer = options.setTimer || setTimeout;
    const clearTimer = options.clearTimer || clearTimeout;
    const elements = [...root.querySelectorAll('[data-tooltip-key]')];
    const removers = [];
    let timer = null;

    const hide = () => {
        if (timer) clearTimer(timer);
        timer = null;
        surface.classList.add('hidden');
    };
    const position = (element) => {
        const rect = element.getBoundingClientRect();
        const width = surface.offsetWidth || 180;
        const height = surface.offsetHeight || 28;
        const left = Math.max(8, Math.min((view.innerWidth || 320) - width - 8, rect.left + rect.width / 2 - width / 2));
        const above = rect.top - height - 8;
        const top = above >= 8 ? above : Math.min((view.innerHeight || 500) - height - 8, rect.bottom + 8);
        surface.style.left = `${left}px`;
        surface.style.top = `${top}px`;
    };
    const show = (element) => {
        surface.textContent = translate(element.dataset.tooltipKey);
        element.setAttribute('aria-describedby', surface.id);
        surface.classList.remove('hidden');
        position(element);
    };

    for (const element of elements) {
        const focus = () => show(element);
        const enter = () => { timer = setTimer(() => show(element), 350); };
        const leave = () => hide();
        const keydown = (event) => { if (event.key === 'Escape') hide(); };
        element.addEventListener('focusin', focus);
        element.addEventListener('focusout', leave);
        element.addEventListener('mouseenter', enter);
        element.addEventListener('mouseleave', leave);
        element.addEventListener('keydown', keydown);
        removers.push(() => {
            element.removeEventListener('focusin', focus);
            element.removeEventListener('focusout', leave);
            element.removeEventListener('mouseenter', enter);
            element.removeEventListener('mouseleave', leave);
            element.removeEventListener('keydown', keydown);
            element.removeAttribute('aria-describedby');
        });
    }

    return () => {
        hide();
        removers.forEach((remove) => remove());
        if (!options.surface) surface.remove();
    };
}
