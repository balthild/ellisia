// Note that it is URI (no scheme and host), not URL
export function resolveRelativeUri(href: string, absolute: string) {
    const base = 'epub:/' + absolute.replace(/^\/*/, '');
    const url = new URL(href, base);
    return url.href.replace(/^epub:\//, '');
}
