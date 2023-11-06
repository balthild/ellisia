import './Toolbar.scss';

import { invoke } from '@tauri-apps/api';
import { createSignal } from 'solid-js';

import { AbstractHistory } from '../history';

export interface ToolbarProps {
    history: AbstractHistory<string>;
}

export function Toolbar(props: ToolbarProps) {
    const openLibrary = async () => {
        const result = await invoke<boolean>('open_library');
        if (result) {
            // TODO: close book?
        }
    };

    const [hasBack, setHasBack] = createSignal(false);
    const [hasForward, setHasForward] = createSignal(false);
    props.history.listen(() => {
        setHasBack(props.history.hasBack());
        setHasForward(props.history.hasForward());
    });

    return (
        <div class="toolbar">
            <button class="btn ri-list-check-2" />
            <button
                class="btn ri-arrow-left-line"
                disabled={!hasBack()}
                onClick={() => props.history.back()}
            />
            <button
                class="btn ri-arrow-right-line"
                disabled={!hasForward()}
                onClick={() => props.history.forward()}
            />
            <div class="sep" />
            <button class="btn ri-database-line" onClick={openLibrary} />
            <div class="flex-spacer" />
            <button class="btn ri-font-size" />
            <button class="btn ri-information-line" />
        </div>
    );
}
