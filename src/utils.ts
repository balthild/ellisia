// Note that the function is for URI (no scheme nor host), not URL.
// `from` is treated as a dir if it ends with `/`, and file if it does not.
export function resolveRelativeUri(href: string, from: string) {
    const base = new URL('epub:/' + from.replace(/^\/*/, ''));
    base.hash = '';
    base.search = '';

    const url = new URL(href, base);
    return url.href.replace(/^epub:\//, '');
}
