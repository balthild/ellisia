import IframeView from 'epubjs/src/managers/views/iframe';

export class IframeViewWithCSP extends IframeView {
    create() {
        const iframe: HTMLIFrameElement = super.create();

        // HTMLIFrameElement.csp is not supported in Firefox and Safari
        // https://developer.mozilla.org/en-US/docs/Web/API/HTMLIFrameElement/csp#browser_compatibility

        return iframe;
    }
}
