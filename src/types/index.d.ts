declare global {
    export const ELLISIA: {
        book: {
            id: string;
            path: string;
        };
        renderer: string;
    };

    export interface EllisiaLibrary {
        books: Record<string, EllisiaLibraryBook>;
    }

    export interface EllisiaLibraryBook {
        path: string;
        location: string;
        last_read_at?: string;
        metadata: {
            unique_id?: string;
            title?: string;
            author?: string;
        }
    }
}

export {}
