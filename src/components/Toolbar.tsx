import './Toolbar.scss';

import { window } from '@tauri-apps/api';
import { JSX } from 'solid-js';

export interface ToolbarProps {
    children: JSX.Element;
    size?: 'large';
}

export function Toolbar(props: ToolbarProps) {
    const dragWindow = (e: MouseEvent) => {
        if (e.target === e.currentTarget && e.button === 0) {
            window.getCurrent().startDragging();
        }
    };

    return (
        <div class="toolbar" classList={{ large: props.size === 'large' }} onMouseDown={dragWindow}>
            {props.children}
        </div>
    );
}
