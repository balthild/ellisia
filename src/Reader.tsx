import './Reader.scss';

import { invoke } from '@tauri-apps/api';
import { createSignal, onCleanup, onMount } from 'solid-js';
import { createStore } from 'solid-js/store';

import { Navigation } from './components/Navigation';
import { Toolbar } from './components/Toolbar';
import { EpubNavItem, EpubSpineItem } from './epub';
import { AbstractHistory } from './history';
import { resolveRelativeUri } from './utils';

export function Reader() {
    const [spine, setSpine] = createStore({
        path: '',
        items: [] as EpubSpineItem[],
    });

    const [contentId, setContentId] = createSignal<string | undefined>(undefined);
    const contentItem = () => {
        let id = contentId();
        return spine.items.find((item) => item.id === id);
    };

    const prevContentItem = () => {
        const currentId = contentItem()?.id;
        if (!currentId) {
            return spine.items[0];
        }

        const currentIndex = spine.items.findIndex((item) => item.id === currentId);
        if (currentIndex === -1) {
            return spine.items[0];
        }

        if (currentIndex === 0) {
            return false;
        }

        return spine.items[currentIndex - 1];
    };
    const nextContentItem = () => {
        const currentId = contentItem()?.id;
        if (!currentId) {
            return spine.items[0];
        }

        const currentIndex = spine.items.findIndex((item) => item.id === currentId);
        if (currentIndex === -1) {
            return spine.items[0];
        }

        if (currentIndex === spine.items.length - 1) {
            return false;
        }

        return spine.items[currentIndex + 1];
    };

    const [nav, setNav] = createStore({
        path: '',
        items: [] as EpubNavItem[],
    });

    const [segmentsHaveBeenRead, setSegmentsHaveBeenRead] = createSignal<string[]>([]);

    let iframe: HTMLIFrameElement | undefined = undefined;
    let iframeLoaded = false;

    // After the iframe is loaded, it will navigate to this href.
    let pendingLocation: string | undefined = undefined;

    const iframePostMessage = (action: string, payload: object) => {
        const message = { action, ...payload };
        iframe?.contentWindow?.postMessage(message, RENDERER);
    };

    const onIframeMessage = (event: MessageEvent) => {
        if (event.origin !== RENDERER) return;

        switch (event.data.action) {
            case 'loaded':
                iframeLoaded = true;
                if (pendingLocation) {
                    syncHistoryLocation(pendingLocation);
                    pendingLocation = undefined;
                }
                break;

            case 'navigate':
                const prefix = `${RENDERER}/book/${BOOK_ID}/`;
                const href = event.data.href.replace(prefix, '');
                history.push(href);
                break;

            case 'toc':
                setSegmentsHaveBeenRead(event.data.segments);
                break;

            case 'progress':
                invoke('save_progress', {
                    id: BOOK_ID,
                    path: contentItem()?.canonicalPath ?? '',
                    progress: event.data.progress,
                });
                break;

            case 'prev':
                let prev = prevContentItem();
                if (prev !== false) {
                    history.push(prev?.canonicalPath);
                }
                break;

            case 'next':
                let next = nextContentItem();
                if (next !== false) {
                    history.push(next?.canonicalPath);
                }
                break;

            default:
                console.log('Unknown Message', event.data);
        }
    };
    onMount(() => {
        window.addEventListener('message', onIframeMessage);
        onCleanup(() => {
            window.removeEventListener('message', onIframeMessage);
        });
    });

    // Notify the iframe that it should navigate
    const syncHistoryLocation = (href: string) => {
        // When the iframe has not loaded yet, we can't navigate it.
        // So we save the href and navigate to it when the iframe is loaded.
        if (!iframeLoaded) {
            pendingLocation = href;
            return;
        }

        iframeLoaded = false;

        let [path, seg] = href.split('#');

        let item = spine.items.find((item) => item.canonicalPath === path);
        if (!item) {
            item = spine.items[0];
            path = item.canonicalPath;
        }

        setContentId(item?.id);

        let url = `${RENDERER}/book/${BOOK_ID}/${path}`;
        if (seg) {
            url += `#${seg}`;
        }

        iframePostMessage('navigate', { url });
    };

    let history = new AbstractHistory({ initialStates: [''] });
    const clearHistoryListener = history.listen((update) => {
        syncHistoryLocation(update.location.state);
    });
    onCleanup(clearHistoryListener);

    onMount(async () => {
        // I really don't want to write the XML data types again. Use the any-way anyway.
        const rootfile = await invoke<any>('get_rootfile', { id: BOOK_ID });
        const toc = await invoke<any>('get_toc', { id: BOOK_ID });

        const spineItems: EpubSpineItem[] = [];
        for (const itemRef of rootfile.package.spine.children) {
            const item = rootfile.package.manifest.children.find((item: any) => item.id === itemRef.idref);
            spineItems.push({
                id: item.id,
                path: item.href,
                canonicalPath: resolveRelativeUri(item.href, rootfile.path),
            });
        }

        // const byPlayOrder = (a: any, b: any) => a.play_order - b.play_order;
        const navItems: EpubNavItem[] = [];
        for (const item of toc.ncx.nav_map.children) {
            navItems.push({
                label: item.nav_label.text,
                href: item.content.src,
                level: 0,
                absoluteHref: resolveRelativeUri(item.content.src, toc.path),
            });
            for (const child of item.children ?? []) {
                navItems.push({
                    label: child.nav_label.text,
                    href: child.content.src,
                    level: 1,
                    absoluteHref: resolveRelativeUri(child.content.src, toc.path),
                });
            }
        }

        setSpine({
            path: rootfile.path,
            items: spineItems,
        });

        setNav({
            path: toc.path,
            items: navItems,
        });

        const [path, progress] = await invoke<any>('get_progress', {
            id: BOOK_ID,
        });
        const href = path ? `${path}#!progress=${progress}` : spineItems[0].canonicalPath;

        history.reset([href]);

        syncHistoryLocation(href);
    });

    return (
        <div id="reader" class="full-size">
            <Toolbar history={history} />

            <div class="main">
                <Navigation
                    items={nav.items}
                    currentContentPath={contentItem()?.path}
                    segmentsHaveBeenRead={segmentsHaveBeenRead()}
                    onNavigate={(href) => history.push(href)}
                />

                <div class="content">
                    <iframe ref={iframe} class="rendered" src={RENDERER} />
                </div>
            </div>
        </div>
    );
}
