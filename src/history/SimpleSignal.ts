export class SimpleSignal<F extends Function> {
    handlers: F[] = [];

    get length() {
        return this.handlers.length;
    }

    push(fn: F) {
        this.handlers.push(fn);

        return () => {
            this.handlers = this.handlers.filter((x) => x !== fn);
        };
    }

    call(arg: any) {
        this.handlers.forEach((fn) => fn(arg));
    }
}
