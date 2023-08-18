import './Library.scss';

import { invoke } from '@tauri-apps/api';
import { createSignal, For, onMount } from 'solid-js';

export function Library() {
    const [library, setLibrary] = createSignal({
        books: {} as Record<string, any>,
    });

    onMount(async () => {
        const library = await invoke<any>('get_library');
        setLibrary(library);
    });

    const [selectedBook, setSelectedBook] = createSignal<string>();

    const [erroredCover, setErroredCover] = createSignal<string[]>([]);

    const openBook = async (book: any) => {
        const result = await invoke<boolean>('open_book', { path: book.path });
        if (result) {
            await invoke<boolean>('close_library')
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
                    <For each={Object.entries(library().books)}>{([id, book]) => (
                        <tr
                            onDblClick={[openBook, book]}
                            onClick={[setSelectedBook, id]}
                            classList={{ selected: selectedBook() === id }}
                        >
                            <td>
                                <img
                                    class="cover cover-thumbnail"
                                    classList={{ errored: erroredCover().includes(id) }}
                                    src={`${RENDERER}/cover/${id}.png`}
                                    onError={() => setErroredCover(xs => [...xs, id])}
                                />
                                <div class='cover cover-text' classList={{ errored: erroredCover().includes(id) }}>
                                    {(book.metadata.title ?? getFilename(book.path))[0]}
                                </div>
                            </td>
                            <td>
                                <div class="title">{book.metadata.title ?? getFilename(book.path)}</div>
                                <div class="author">{book.metadata.author}</div>
                            </td>
                            <td class="nowrap">{new Date(book.last_read_at * 1000).toLocaleString()}</td>
                        </tr>
                    )}</For>
                </tbody>
            </table>
        </div>
    );
}

function getFilename(path: string) {
    let url = new URL(`file://${path}`);
    const filename = url.pathname.split("/").pop();
    return decodeURIComponent(filename!);
}
