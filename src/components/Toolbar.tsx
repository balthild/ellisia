import './Toolbar.scss';

export function Toolbar() {
    return (
        <div class="toolbar">
            <button class="btn ri-list-check-2" />
            <button class="btn ri-arrow-left-line" />
            <button class="btn ri-arrow-right-line" />
            <div class="flex-spacer" />
            <button class="btn ri-font-size" />
            <button class="btn ri-information-line" />
        </div>
    )
}
