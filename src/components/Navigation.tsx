import './Navigation.scss';

import { OverlayScrollbarsComponent } from 'overlayscrollbars-solid';
import { createMemo, For } from 'solid-js';

import { EpubNavItem } from '../epub';

export interface NavProps {
    items: EpubNavItem[];
    currentContentPath: string | undefined;
    segmentsHaveBeenRead: string[];
    onNavigate: (href: string) => void;
}

export function Navigation(props: NavProps) {
    const currentNavHref = createMemo(() => {
        if (!props.currentContentPath) return;

        if (props.segmentsHaveBeenRead.length === 0) {
            return props.currentContentPath;
        }

        for (let i = props.items.length - 1; i >= 0; i--) {
            const [path, segment] = props.items[i].href.split('#');
            if (path !== props.currentContentPath) continue;

            for (let j = props.segmentsHaveBeenRead.length - 1; j >= 0; j--) {
                if (props.segmentsHaveBeenRead[j] === segment) {
                    return props.items[i].href;
                }
            }
        }

        return props.currentContentPath;
    });

    return (
        <OverlayScrollbarsComponent
            class="toc"
            options={{
                overflow: { x: 'hidden', y: 'scroll' },
                scrollbars: { autoHide: 'scroll' },
            }}
        >
            <For each={props.items}>{(item) => (
                <div
                    class="toc-item"
                    classList={{
                        current: item.href === currentNavHref(),
                        child: item.level == 1,
                        hidden: item.level > 1,
                    }}
                    onClick={() => props.onNavigate(item.canonicalPath)}
                >
                    {item.label}
                </div>
            )}</For>
        </OverlayScrollbarsComponent >
    );
}
