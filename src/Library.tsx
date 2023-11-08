import './Library.scss';

import { dialog, invoke } from '@tauri-apps/api';
import { createResource, createSignal, For, onCleanup } from 'solid-js';

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

    const booksSorted = () => {
        let data = library.latest;
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
    };

    const [selectedBook, setSelectedBook] = createSignal<string>();

    const [erroredCover, setErroredCover] = createSignal<string[]>([]);

    const addBook = async () => {
        const path = await dialog.open({
            filters: [{ name: 'Epub', extensions: ['epub'] }],
        });
        if (!path) {
            return;
        }

        console.log(path);
        const result = await invoke<boolean>('open_book', { path });
        if (result) {
            refetch();
        }
    };

    const openBook = async (book: EllisiaLibraryBook) => {
        const result = await invoke<boolean>('open_book', { path: book.path });
        if (result) {
            await invoke<boolean>('close_library');
        }
    };

    return (
        <div id="library">
            <Toolbar size="large">
                <ToolbarIcon
                    bordered
                    onClick={addBook}
                    icon="folder-open-line"
                    label="Open"
                />
            </Toolbar>

            <table class="books">
                <thead>
                    <tr>
                        <th></th>
                        <th>Title</th>
                        <th>Last Read</th>
                    </tr>
                </thead>
                <tbody>
                    <For each={booksSorted()}>
                        {(book) => (
                            <tr
                                onDblClick={[openBook, book]}
                                onClick={[setSelectedBook, book.id]}
                                classList={{ selected: selectedBook() === book.id }}
                            >
                                <td>
                                    <img
                                        class="cover cover-thumbnail"
                                        classList={{ errored: erroredCover().includes(book.id) }}
                                        src={`${ELLISIA.renderer}/cover/${book.id}.png`}
                                        onError={() => setErroredCover((xs) => [...xs, book.id])}
                                    />
                                    <div
                                        class="cover cover-text"
                                        classList={{ errored: erroredCover().includes(book.id) }}
                                    >
                                        {(book.metadata.title ?? getFilename(book.path))[0]}
                                    </div>
                                </td>
                                <td>
                                    <div class="title">
                                        {book.metadata.title ?? getFilename(book.path)}
                                    </div>
                                    <div class="author">{book.metadata.author}</div>
                                </td>
                                <td class="nowrap">{book.last_read_at?.toLocaleString()}</td>
                            </tr>
                        )}
                    </For>
                </tbody>
            </table>
        </div>
    );
}

function getFilename(path: string) {
    let url = new URL(`file://${path}`);
    const filename = url.pathname.split('/').pop();
    return decodeURIComponent(filename!);
}
