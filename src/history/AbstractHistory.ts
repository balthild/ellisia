import { SimpleSignal } from './SimpleSignal';
import { Action, Blocker, Entry, Listener, Options } from './types';
import { clamp, readOnly } from './utils';

/**
 * A history is an interface to the navigation stack. The history serves as the
 * source of truth for the current location, as well as provides a set of
 * methods that may be used to change it.
 *
 * An abstract history stores states without URLs in memory. This is useful in
 * stateful environments where there is no web browser, such as client apps.
 *
 * It is similar to the DOM's `window.history` object, but with a smaller, more
 * focused API.
 */
export class AbstractHistory<T> {
    protected entries: Entry<T>[];

    protected index: number;

    protected listeners: SimpleSignal<Listener<T>>;

    protected blockers: SimpleSignal<Blocker<T>>;

    /**
     * The current location. This value is mutable.
     */
    public get location(): Entry<T> {
        return this.entries[this.index];
    }

    /**
     * Creates a new history instance. The current state is determined by the
     * initial states.
     *
     * @param options
     */
    constructor(options: Options<T> = {}) {
        let { initialStates = [], initialIndex } = options;

        this.listeners = new SimpleSignal<Listener<T>>();
        this.blockers = new SimpleSignal<Blocker<T>>();

        if (initialStates.length === 0) {
            throw new Error('initialStates must contain at least one entry');
        }

        if (initialIndex == null) {
            initialIndex = initialStates.length - 1;
        } else {
            initialIndex = clamp(initialIndex, 0, initialStates.length - 1);
        }

        this.entries = initialStates.map(AbstractHistory.createEntry<T>);
        this.index = initialIndex;
    }

    /**
     * Triggers all blockers with the given navigation. This is called before
     * committing the navigation.
     *
     * If there exists any blocker, this function returns `false` and the
     * navigation should be canceled.
     *
     * @param action
     * @param location
     * @param retry
     * @returns
     */
    protected callBlockers(action: Action, location: Entry<T>, retry: () => void) {
        if (!this.blockers.length) {
            return true;
        }

        this.blockers.call({ action, location, retry });

        return false;
    }

    /**
     * Triggers all listeners with the given navigation. This is called after
     * committing the navigation.
     *
     * @param action
     * @param location
     */
    protected callListeners(action: Action, location: Entry<T>) {
        this.listeners.call({ action, location });
    }

    /**
     * Pushes a new location onto the history stack, increasing its length by one.
     * If there were any entries in the stack after the current one, they are
     * lost.
     *
     * @param to - The new URL
     * @param state - Data to associate with the new location
     */
    public push(state: T) {
        const action = Action.Push;
        const location = AbstractHistory.createEntry<T>(state);
        const retry = () => this.push(state);

        if (this.callBlockers(action, location, retry)) {
            this.index += 1;
            this.entries.splice(this.index, this.entries.length, location);
            this.callListeners(action, location);
        }
    }

    /**
     * Replaces the current location in the history stack with a new one.  The
     * location that was replaced will no longer be available.
     *
     * @param to - The new URL
     * @param state - Data to associate with the new location
     */
    public replace(state: T) {
        const action = Action.Replace;
        const location = AbstractHistory.createEntry<T>(state);
        const retry = () => this.replace(state);

        if (this.callBlockers(action, location, retry)) {
            this.entries[this.index] = location;
            this.callListeners(action, location);
        }
    }

    /**
     * Replaces the entire history stack to the provided states.
     *
     * @param initialStates
     * @param initialIndex
     */
    public reset(initialStates: T[], initialIndex?: number) {
        if (initialStates.length === 0) {
            throw new Error('initialStates must contain at least one entry');
        }

        if (initialIndex == null) {
            initialIndex = initialStates.length - 1;
        } else {
            initialIndex = clamp(initialIndex, 0, initialStates.length - 1);
        }

        const action = Action.Replace;
        const location = AbstractHistory.createEntry<T>(initialStates[initialIndex]);
        const retry = () => this.reset(initialStates, initialIndex);

        if (this.callBlockers(action, location, retry)) {
            this.entries = initialStates.map(AbstractHistory.createEntry<T>);
            this.index = initialIndex;
            this.callListeners(action, location);
        }
    }

    /**
     * Navigates `n` entries backward/forward in the history stack relative to the
     * current index. For example, a "back" navigation would use go(-1).
     *
     * @param delta - The delta in the stack index
     */
    public go(delta: number) {
        const index = clamp(this.index + delta, 0, this.entries.length - 1);
        const action = Action.Pop;
        const location = this.entries[index];
        const retry = () => this.go(delta);

        if (this.callBlockers(action, location, retry)) {
            this.index = index;
            this.callListeners(action, location);
        }
    }

    /**
     * Navigates to the previous entry in the stack. Identical to go(-1).
     *
     * Warning: if the current location is the first location in the stack, this
     * will unload the current document.
     */
    public back() {
        this.go(-1);
    }

    /**
     * Navigates to the next entry in the stack. Identical to go(1).
     */
    public forward() {
        this.go(1);
    }

    /**
     * Check if the current location is at the top of the history stack.
     *
     * @returns hasBack
     */
    public hasBack() {
        return this.index > 0;
    }

    /**
     * Check if the current location is at the bottom of the history stack.
     *
     * @returns hasForward
     */
    public hasForward() {
        return this.index < this.entries.length - 1;
    }

    /**
     * Sets up a listener that will be called whenever the current location
     * changes.
     *
     * @param listener - A function that will be called when the location changes
     * @returns unlisten - A function that may be used to stop listening
     */
    public listen(listener: Listener<T>) {
        return this.listeners.push(listener);
    }

    /**
     * Prevents the current location from changing and sets up a listener that
     * will be called instead.
     *
     * @param blocker - A function that will be called when a transition is blocked
     * @returns unblock - A function that may be used to stop blocking
     */
    public block(blocker: Blocker<T>) {
        return this.blockers.push(blocker);
    }

    /**
     * Creates a location entry with a unique key.
     *
     * @param state
     * @returns entry
     */
    protected static createEntry<T>(state: T) {
        return readOnly<Entry<T>>({
            state,
            key: Math.random().toString(36).substring(2, 10),
        });
    }
}
