import './Reader.scss';

import { invoke } from '@tauri-apps/api';
import { Book, Contents, EpubCFI, Location } from 'epubjs';
import { createSignal, onCleanup } from 'solid-js';

import { Navigation, TocItem } from './components/Navigation';
import { Toolbar } from './components/Toolbar';
import { AbstractHistory } from './history';

export function Reader() {
    let history = new AbstractHistory({ initialStates: [''] });
    history.listen((update) => {
        // Auto sync only for back/forward. Ignore push/replace.
        if (update.action === 'POP') {
            renderTarget(update.location.state);
        }
    });

    const navigate = (state: string) => {
        history.push(state);
        renderTarget(state);
    };

    let book = new Book(`${RENDERER}/book/${BOOK_ID}/`);

    const getCfiFromHref = async (href: string) => {
        const [path, segment] = href.split('#');

        const section = book.spine.get(path);
        await section.load();

        const element = section.document.getElementById(segment);

        return section.cfiFromElement(element ?? section.document.body);
    };

    const [toc, setToc] = createSignal<TocItem[]>([]);
    const [tocCurrentId, setTocCurrentId] = createSignal<string>();

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
        setToc(items);
    };

    const calcCurrentTocItem = (location: Location) => {
        const cfi = location.start.cfi;
        const path = book.canonical(location.start.href);

        let lastItemAbove = undefined;
        for (const item of toc()) {
            if (!book.canonical(item.href).startsWith(path)) {
                continue;
            }

            if (EpubCFI.prototype.compare(item.cfi, cfi) > 0) {
                break;
            }

            lastItemAbove = item;
        }

        setTocCurrentId(lastItemAbove?.id);
    };

    const renderTarget = (target: string) => {
        requestIdleCallback(() => {
            const rendition = book.rendition;
            if (!rendition) return;
            rendition.display(target).catch(() => rendition.display());
        });
    };

    const attachEpubView = async (element: HTMLElement) => {
        const rendition = book.renderTo(element, {
            flow: 'scrolled-doc',
            width: '100%',
            height: '100%',
        });

        await book.ready;
        await initializeToc();

        rendition.on('relocated', (location: Location) => {
            calcCurrentTocItem(location);
            history.replace(location.start.cfi);
            invoke('save_progress', {
                id: BOOK_ID,
                location: location.start.cfi
            });
        });

        rendition.hooks.content.register((contents: Contents) => {
            contents.on('linkClicked', (href: string) => {
                const relative = book.path.relative(href);
                history.push(relative);
			});

            contents.document.addEventListener('mouseup', (event) => {
                // Mouse side buttons
                switch (event.button) {
                    case 3:
                        history.back();
                        break;
                    case 4:
                        history.forward();
                        break;
                }
            });
        });

        const location = await invoke<any>('get_progress', { id: BOOK_ID });
        history.reset([location]);
        renderTarget(location);

        onCleanup(() => book?.destroy());
    };

    return (
        <div id="reader" class="full-size">
            <Toolbar history={history} />

            <div class="main">
                <Navigation
                    items={toc()}
                    currentId={tocCurrentId()}
                    onNavigate={navigate}
                />

                <div class="content">
                    <div ref={attachEpubView} class="rendered" />
                </div>
            </div>
        </div>
    );
}
