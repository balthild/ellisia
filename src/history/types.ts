export type Options<T> = {
    initialStates?: T[];
    initialIndex?: number;
};

/**
 * Actions represent the type of change to a location value.
 *
 * @see https://github.com/remix-run/history/tree/main/docs/api-reference.md#action
 */
export enum Action {
    /**
     * A POP indicates a change to an arbitrary index in the history stack, such
     * as a back or forward navigation. It does not describe the direction of the
     * navigation, only that the current index changed.
     *
     * Note: This is the default action for newly created history objects.
     */
    Pop = "POP",

    /**
     * A PUSH indicates a new entry being added to the history stack, such as when
     * a link is clicked and a new page loads. When this happens, all subsequent
     * entries in the stack are lost.
     */
    Push = "PUSH",

    /**
     * A REPLACE indicates the entry at the current index in the history stack
     * being replaced by a new one.
     */
    Replace = "REPLACE",
}

/**
 * A unique string associated with a location. May be used to safely store
 * and retrieve data in some other storage API, like `localStorage`.
 *
 * @see https://github.com/remix-run/history/tree/main/docs/api-reference.md#location.key
 */
export type Key = string;

/**
 * An entry in a history stack. A location contains information about the
 * URL path, as well as possibly some arbitrary state and a key.
 *
 * @see https://github.com/remix-run/history/tree/main/docs/api-reference.md#location
 */
export interface Entry<T> {
    /**
     * A value of arbitrary data associated with this location.
     *
     * @see https://github.com/remix-run/history/tree/main/docs/api-reference.md#location.state
     */
    state: T;

    /**
     * A unique string associated with this location. May be used to safely store
     * and retrieve data in some other storage API, like `localStorage`.
     *
     * Note: This value is always "default" on the initial location.
     *
     * @see https://github.com/remix-run/history/tree/main/docs/api-reference.md#location.key
     */
    key: Key;
}

/**
 * A change to the current location.
 */
export interface Update<T> {
    /**
     * The action that triggered the change.
     */
    action: Action;

    /**
     * The new location.
     */
    location: Entry<T>;
}

/**
 * A function that receives notifications about location changes.
 */
export interface Listener<T> {
    (update: Update<T>): void;
}

/**
 * A change to the current location that was blocked. May be retried
 * after obtaining user confirmation.
 */
export interface Transition<T> extends Update<T> {
    /**
     * Retries the update to the current location.
     */
    retry(): void;
}

/**
 * A function that receives transitions when navigation is blocked.
 */
export interface Blocker<T> {
    (tx: Transition<T>): void;
}
