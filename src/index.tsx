/// <reference path="./index.d.ts" />

/* @refresh reload */
import './styles.scss';

import { render } from 'solid-js/web';

import { Reader } from './Reader';

render(
    () => <Reader />,
    document.getElementById("app") as HTMLElement,
);

document.addEventListener('contextmenu', (event) => event.preventDefault());
