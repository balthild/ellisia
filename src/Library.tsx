import './Library.scss';

import { dialog, invoke } from '@tauri-apps/api';
import { createEffect, createResource, createSignal, For, onCleanup } from 'solid-js';
import { createStore, reconcile } from 'solid-js/store';

import { Toolbar } from './components/Toolbar';
import { ToolbarIcon } from './components/ToolbarIcon';

export function Library() {
    const [library, { refetch }] = createResource(() => invoke<EllisiaLibrary>('get_library'));

    const refetchHandler = setInterval(refetch, 60000);
    window.addEventListener('focus', refetch);
    onCleanup(() => {
        clearInterval(refetchHandler);
        window.removeEventListener('focus', refetch);
    });

    const [books, setBooks] = createStore(transformBooks());
    createEffect(() => setBooks(reconcile(transformBooks(library.latest))));

    const [selectedBookId, setSelectedBookId] = createSignal<string>();

    const [erroredCover, setErroredCover] = createSignal<string[]>([]);

    const deselectBook = (event: MouseEvent) => {
        if (event.target === event.currentTarget) {
            setSelectedBookId(undefined);
        }
    };

    const openNewBook = async () => {
        const path = await dialog.open({
            filters: [{ name: 'Epub', extensions: ['epub'] }],
        });
        if (!path) {
            return;
        }

        const result = await invoke<boolean>('open_book', { path });
        if (result) {
            refetch();
        }
    };

    const openBook = async (path: string) => {
        const result = await invoke<boolean>('open_book', { path });
        if (result) {
            await invoke<boolean>('close_library');
        }
    };

    return (
        <div id="library">
            <Toolbar size="large">
                <ToolbarIcon bordered onClick={openNewBook} icon="folder-open-line" label="Open" />
            </Toolbar>

            <div class="books" onClick={deselectBook}>
                <For each={books}>
                    {(book) => (
                        <div
                            class="book"
                            classList={{ selected: selectedBookId() === book.id }}
                            onDblClick={[openBook, book.path]}
                            onClick={[setSelectedBookId, book.id]}
                        >
                            {erroredCover().includes(book.id) ? (
                                <div class="cover cover-text">{book.metadata.title?.[0]}</div>
                            ) : (
                                <img
                                    class="cover cover-thumbnail"
                                    src={`${ELLISIA.renderer}/cover/${book.id}.png`}
                                    onError={() => setErroredCover((xs) => [...xs, book.id])}
                                />
                            )}

                            <div>
                                <div class="title">
                                    {book.metadata.title ?? getFilename(book.path)}
                                </div>
                                <div class="author">{book.metadata.author}</div>
                                <div class="last-read">
                                    Last Read:
                                    <br />
                                    {book.last_read_at?.toLocaleString()}
                                </div>
                            </div>
                        </div>
                    )}
                </For>
            </div>
        </div>
    );
}

function transformBooks(data?: EllisiaLibrary) {
    if (!data) {
        return [];
    }

    const books = Object.entries(data.books).map(([id, book]) => {
        let last_read_at = undefined;
        if (book.last_read_at) {
            last_read_at = new Date(book.last_read_at);
        }
        return {
            ...book,
            id,
            last_read_at,
        };
    });

    books.sort((a, b) => {
        // NaN always goes to the end
        const timeA = b.last_read_at?.getTime() ?? NaN;
        const timeB = a.last_read_at?.getTime() ?? NaN;
        return timeA - timeB;
    });

    return books;
}

function getFilename(path: string) {
    let url = new URL(`file://${path}`);
    const filename = url.pathname.split('/').pop();
    return decodeURIComponent(filename!);
}
