import './Toolbar.scss';

import { invoke } from '@tauri-apps/api';

export function Toolbar() {
    const openLibrary = async () => {
        const result = await invoke<boolean>('open_library');
        if (result) {
            // TODO: close book?
        }
    };

    return (
        <div class="toolbar">
            <button class="btn ri-list-check-2" />
            <button class="btn ri-arrow-left-line" />
            <button class="btn ri-arrow-right-line" />
            <div class="sep" />
            <button class="btn ri-database-line" onClick={openLibrary} />
            <div class="flex-spacer" />
            <button class="btn ri-font-size" />
            <button class="btn ri-information-line" />
        </div>
    );
}
