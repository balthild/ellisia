import './Navigation.scss';

import { NavItem } from 'epubjs';
import { OverlayScrollbarsComponent } from 'overlayscrollbars-solid';
import { For } from 'solid-js';

export interface TocItem extends NavItem {
    level: number;
    cfi: string;
}

export interface NavProps {
    items: TocItem[];
    current: TocItem | undefined;
    onNavigate: (href: string) => void;
}

export function Navigation(props: NavProps) {
    return (
        <OverlayScrollbarsComponent
            class="toc"
            options={{
                overflow: { x: 'hidden', y: 'scroll' },
                scrollbars: { autoHide: 'scroll' },
            }}
        >
            <For each={props.items}>
                {(item) => (
                    <div
                        class="toc-item"
                        classList={{
                            current: item.id === props.current?.id,
                            child: item.level == 1,
                            hidden: item.level > 1,
                        }}
                        onClick={() => props.onNavigate(item.href)}
                    >
                        {item.label}
                    </div>
                )}
            </For>
        </OverlayScrollbarsComponent>
    );
}
