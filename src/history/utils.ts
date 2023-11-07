export function clamp(n: number, lowerBound: number, upperBound: number) {
    if (n < lowerBound) return lowerBound;
    if (n > upperBound) return upperBound;
    return n;
}

const __DEV__ = process.env.NODE_ENV !== 'production';
export const readOnly: <T>(obj: T) => Readonly<T> = __DEV__
    ? (obj) => Object.freeze(obj)
    : (obj) => obj;
