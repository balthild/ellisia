import './ToolbarIcon.scss';

import { Show } from 'solid-js';

export interface ToolbarIconProps {
    bordered?: boolean;
    icon?: string;
    label?: string;
    disabled?: boolean;
    onClick?: () => void;
}

export function ToolbarIcon(props: ToolbarIconProps) {
    return (
        <button
            class="btn"
            classList={{
                bordered: props.bordered,
                square: !!props.icon && !props.label,
            }}
            disabled={props.disabled}
            onClick={props.onClick}
        >
            <Show when={props.icon}>
                <span class={`icon ri-${props.icon}`}></span>
            </Show>
            <Show when={props.label}>
                <span class="label">{props.label}</span>
            </Show>
        </button>
    );
}
