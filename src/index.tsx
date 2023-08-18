/// <reference path="./index.d.ts" />

/* @refresh reload */
import './styles.scss';

import { render } from 'solid-js/web';

import { Library } from './Library';
import { Reader } from './Reader';

render(
    () => BOOK_ID ? <Reader /> : <Library />,
    document.getElementById("app") as HTMLElement,
);

document.addEventListener('contextmenu', (event) => event.preventDefault());
