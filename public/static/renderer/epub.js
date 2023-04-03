const ELLISIA_META = document.querySelector('meta[name="ellisia"]')?.content;
const ELLISIA_BLANK_PAGE = ELLISIA_META === 'blank';

function scrollToHash() {
    const hash = location.hash.slice(1);
    if (!hash) {
        window.scrollTo(0, 0);
        return;
    }

    if (hash.startsWith('!progress=')) {
        const progress = parseFloat(hash.slice(10));
        if (progress && !isNaN(progress)) {
            const height = document.body.scrollHeight - window.innerHeight;
            window.scrollTo(0, height * progress);
        }
        return;
    }

    const element = document.getElementById(hash);
    if (!element) {
        window.scrollTo(0, 0);
        return;
    }

    // Scroll to the element with an offset of 1/4 window height
    const rect = element.getBoundingClientRect();
    const position = window.pageYOffset + rect.top;
    const offset = window.innerHeight / 4;
    window.scrollTo(0, position - offset);
}

function postAppMessage(action, payload) {
    window.parent.postMessage({ action, ...payload }, '*');
}

function reportLoaded() {
    postAppMessage('loaded');
}

let segmentsPositions = [];
function cacheSegmentsPositions() {
    const start = performance.now();

    const elements = Array.from(document.querySelectorAll('[id]'));
    segmentsPositions = elements.map((element) => {
        const rect = element.getBoundingClientRect();
        return {
            id: element.id,
            position: rect.top + window.pageYOffset,
        }
    });

    const end = performance.now();
}

let lastNearestSegmentId = undefined;
function reportTocNavigation() {
    if (ELLISIA_BLANK_PAGE) return;

    if (segmentsPositions.length === 0) return;

    let segments = segmentsPositions.filter((item) => {
        return item.position < window.pageYOffset + window.innerHeight / 2;
    });

    const nearest = segments[segments.length - 1];
    if (nearest?.id === lastNearestSegmentId) return;
    lastNearestSegmentId = nearest?.id;

    postAppMessage('toc', {
        segments: segments.map((item) => item.id),
    });
}

function reportProgress() {
    if (ELLISIA_BLANK_PAGE) return;

    const height = document.body.scrollHeight - window.innerHeight;
    const progress = window.pageYOffset / height;
    postAppMessage('progress', { progress });
}

function onNavigated() {
    reportLoaded();
    reportTocNavigation();
    reportProgress();
}

function navigateInSamePage() {
    scrollToHash();
    onNavigated();
}

function handleMessages(event) {
    switch (event.data.action) {
        case 'navigate':
            // If the entire URL is not changed, location.replace() will do nothing
            if (event.data.url === location.href) {
                navigateInSamePage();
                break;
            }

            // If only the hash is changed, 'hashchange' event will be emitted
            // If the part before hash is changed, page will be reloaded
            location.replace(event.data.url);
            break;
    }
}

document.addEventListener('contextmenu', (event) => event.preventDefault());

window.addEventListener('DOMContentLoaded', () => {
    window.addEventListener('hashchange', navigateInSamePage);

    document.querySelectorAll('a').forEach((element) => {
        if (element.classList.contains('ellisia')) return;

        element.addEventListener('click', (event) => {
            event.preventDefault();
            const href = event.currentTarget.href?.trim();
            if (href && !href.startsWith('javascript:')) {
                postAppMessage('navigate', { href });
            }
        });
    });

    document.querySelectorAll('a.ellisia-prev').forEach((element) => {
        element.addEventListener('click', (event) => {
            event.preventDefault();
            postAppMessage('prev');
        });
    });
    document.querySelectorAll('a.ellisia-next').forEach((element) => {
        element.addEventListener('click', (event) => {
            event.preventDefault();
            postAppMessage('next');
        });
    });

    window.addEventListener('message', handleMessages);
});

window.addEventListener('load', () => {
    cacheSegmentsPositions();
    if (segmentsPositions.length === 0) {
        postAppMessage('toc', { segments: [] });
    }

    scrollToHash();

    window.addEventListener('scroll', _.throttle(reportProgress, 1000), { passive: true });
    window.addEventListener('scroll', _.throttle(reportTocNavigation, 100), { passive: true });

    window.addEventListener(
        'resize',
        _.throttle(() => requestIdleCallback(cacheSegmentsPositions), 500),
        { passive: true },
    );

    onNavigated();

    document.body.classList.add('ellisia-loaded');
});
