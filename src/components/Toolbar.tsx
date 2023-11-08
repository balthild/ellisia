import './Toolbar.scss';

import { JSX } from 'solid-js';

export interface ToolbarProps {
    children: JSX.Element;
    size?: 'large';
}

export function Toolbar(props: ToolbarProps) {
    return (
        <div class="toolbar" classList={{ large: props.size === 'large' }}>
            {props.children}
        </div>
    );
}
