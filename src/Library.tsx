import './Library.scss';

import { invoke } from '@tauri-apps/api';
import { createSignal, For, onMount } from 'solid-js';

export function Library() {
    const [library, setLibrary] = createSignal({
        books: {} as Record<string, any>,
    });

    const booksSorted = () => {
        const books = Object.entries(library().books).map(([id, book]) => {
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
            return b.last_read_at?.getTime() - a.last_read_at?.getTime();
        });
        return books;
    };

    onMount(async () => {
        const library = await invoke<any>('get_library');
        setLibrary(library);
    });

    const [selectedBook, setSelectedBook] = createSignal<string>();

    const [erroredCover, setErroredCover] = createSignal<string[]>([]);

    const openBook = async (book: any) => {
        const result = await invoke<boolean>('open_book', { path: book.path });
        if (result) {
            await invoke<boolean>('close_library');
        }
    };

    return (
        <div id="library">
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
                                <td class="nowrap">{book.last_read_at.toLocaleString()}</td>
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
