import './Reader.scss';

import { invoke } from '@tauri-apps/api';
import { Book, Contents, EpubCFI, Location } from 'epubjs';
import { createSignal, onCleanup } from 'solid-js';

import BODY_END from './assets/body-end.html?raw';
import BODY_START from './assets/body-start.html?raw';
import CSP from './assets/csp.txt?raw';
import HEAD_END from './assets/head-end.html?raw';
import { Navigation, TocItem } from './components/Navigation';
import { Toolbar } from './components/Toolbar';
import { ToolbarIcon } from './components/ToolbarIcon';
import { IframeViewWithCSP } from './epubjs/IframeViewWithCSP';
import { AbstractHistory } from './history';

export function Reader() {
    const openLibrary = async () => {
        const result = await invoke<boolean>('open_library');
        // TODO: close book?
    };

    const [hasBack, setHasBack] = createSignal(false);
    const [hasForward, setHasForward] = createSignal(false);

    let history = new AbstractHistory({ initialStates: [''] });
    history.listen((update) => {
        // Auto sync only for back/forward. Ignore push/replace.
        if (update.action === 'POP') {
            displaySection(update.location.state);
        }

        setHasBack(history.hasBack());
        setHasForward(history.hasForward());
    });

    const navigate = (state: string) => {
        history.push(state);
        displaySection(state);
    };

    let book = new Book(`${ELLISIA.renderer}/book/${ELLISIA.book.id}/`);
    onCleanup(() => book.destroy());

    const getCfiFromHref = async (href: string) => {
        const [path, segment] = href.split('#');

        const section = book.spine.get(path);
        await section.load();

        const element = section.document.getElementById(segment);

        return section.cfiFromElement(element ?? section.document.body);
    };

    const [tocItems, setTocItems] = createSignal<TocItem[]>([]);
    const [currentTocItem, setCurrentTocItem] = createSignal<TocItem>();

    const initializeToc = async () => {
        const items = [];
        for (const item of book.navigation.toc) {
            items.push({
                ...item,
                level: 0,
                cfi: await getCfiFromHref(item.href),
            });
            for (const subitem of item.subitems ?? []) {
                items.push({
                    ...subitem,
                    level: 1,
                    cfi: await getCfiFromHref(subitem.href),
                });
            }
        }
        setTocItems(items);
    };

    const calcCurrentTocItem = (location: Location) => {
        const cfi = location.start.cfi;
        const path = book.canonical(location.start.href);

        let lastItemAbove = undefined;
        for (const item of tocItems()) {
            if (!book.canonical(item.href).startsWith(path)) {
                continue;
            }

            if (EpubCFI.prototype.compare(item.cfi, cfi) > 0) {
                break;
            }

            lastItemAbove = item;
        }

        setCurrentTocItem(lastItemAbove);
    };

    const displaySection = (target: string) => {
        requestIdleCallback(() => {
            const rendition = book.rendition;
            if (!rendition) return;
            rendition.display(target).catch(() => rendition.display());
        });
    };

    const attachEpubView = async (element: HTMLElement) => {
        const rendition = book.renderTo(element, {
            flow: 'scrolled-doc',
            width: 640,
            height: '100%',
            overflow: 'hidden scroll',
            view: IframeViewWithCSP,
        });

        await book.ready;

        // Process document before it is rendered to iframe
        book.spine.hooks.content.register((document: Document) => {
            const assets = import.meta.env.DEV ? 'http://localhost:1420' : ELLISIA.renderer;
            document.head.insertAdjacentHTML('beforeend', HEAD_END.replaceAll('{assets}', assets));
            document.body.insertAdjacentHTML('afterbegin', BODY_START);
            document.body.insertAdjacentHTML('beforeend', BODY_END);

            document.head
                .querySelectorAll('meta[http-equiv="Content-Security-Policy"]')
                .forEach((element) => element.remove());

            const origins = `${ELLISIA.renderer} ${assets}`;
            const csp = document.createElement('meta');
            csp.setAttribute('http-equiv', 'Content-Security-Policy');
            csp.setAttribute('content', CSP.replaceAll('{origins}', origins));
            document.head.appendChild(csp);
        });

        rendition.on('relocated', (location: Location) => {
            calcCurrentTocItem(location);
            history.replace(location.start.cfi);
            invoke('save_progress', {
                id: ELLISIA.book.id,
                location: location.start.cfi,
            });
        });

        // Process document after it is rendered to iframe
        rendition.hooks.content.register((contents: Contents) => {
            contents.on('linkClicked', (href: string) => {
                const relative = book.path.relative(href);
                history.push(relative);
            });

            contents.document.querySelectorAll('a.ellisia-prev').forEach((element) => {
                element.addEventListener('click', (event) => {
                    event.preventDefault();
                    rendition.prev();
                });
            });
            contents.document.querySelectorAll('a.ellisia-next').forEach((element) => {
                element.addEventListener('click', (event) => {
                    event.preventDefault();
                    rendition.next();
                });
            });

            contents.window.addEventListener('mouseup', (event) => {
                // Mouse side buttons
                if (event.button === 3 || event.button === 4) {
                    event.preventDefault();
                    event.stopPropagation();
                }
                if (event.button === 3 && history.hasBack()) {
                    history.back();
                }
                if (event.button === 4 && history.hasForward()) {
                    history.forward();
                }
            });

            contents.document.addEventListener('contextmenu', (event) => event.preventDefault());

            contents.document.body.classList.add('ellisia-loaded');

            setTimeout(() => {
                const walker = contents.document.createTreeWalker(
                    document.body,
                    NodeFilter.SHOW_ELEMENT
                );
                let element;
                while ((element = walker.nextNode() as Element)) {
                    const style = window.getComputedStyle(element);
                    if (style.fontStyle === 'italic') {
                        element.classList.add('ellisia-emphasis');
                    }
                }
            }, 20);
        });

        // This will trigger `book.spine.hooks.content`, so it needs to be placed after that.
        await initializeToc();

        const location = await invoke<any>('get_progress', { id: ELLISIA.book.id });
        history.reset([location]);
        displaySection(location);
    };

    return (
        <div id="reader" class="full-size">
            <Toolbar>
                <ToolbarIcon icon="list-check-2" />
                <ToolbarIcon
                    disabled={!hasBack()}
                    onClick={() => history.back()}
                    icon="arrow-left-line"
                />
                <ToolbarIcon
                    disabled={!hasForward()}
                    onClick={() => history.forward()}
                    icon="arrow-right-line"
                />
                <div class="sep" />
                <ToolbarIcon icon="database-line" onClick={openLibrary} />
                <div class="flex-spacer" />
                <ToolbarIcon icon="font-size" />
                <ToolbarIcon icon="information-line" />
            </Toolbar>

            <div class="main">
                <Navigation items={tocItems()} current={currentTocItem()} onNavigate={navigate} />

                <div class="content">
                    <div ref={attachEpubView} class="rendered" />
                </div>
            </div>
        </div>
    );
}
