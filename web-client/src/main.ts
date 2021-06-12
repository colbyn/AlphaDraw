// console.log("hello world")

// class DrawCanvas extends HTMLCanvasElement {
//     constructor() {
//         super();
//         this.setAttribute('draw-canvas', '');
//     }
// }

// window.customElements.define('draw-canvas', DrawCanvas, {extends: "canvas"});


// @ts-ignore
import init, { return_value } from '/pkg/rust_runtime.js';
console.log(init)

// init.then(() => {
//     const result = return_value();
//     console.log(`RESULT ${result}`);
// });


// window.onload = () => {
//     // @ts-ignore
//     // import init, { return_value } from '/pkg/rust_runtime.js';
    
//     // init.then(() => {
//     //     const result = return_value();
//     //     console.log(`RESULT ${result}`);
//     // });
// };


