// Note that a dynamic `import` statement here is required due to
// webpack/webpack#6615, but in theory `import { greet } from './pkg';`
// will work here one day as well!
// const rust = import('./pkg');

interface Tag {
    tag: string,
    attrs: Object,
    apply?: (element: Element) => void,
    children: Array<Html>
}

type Html = Tag | string;

function instanceOfElement(object: any): object is Tag {
    if (!instanceOfText(object)) {
        return 'tag' in object;
    } else {
        return false;
    }
}
function instanceOfText(object: any): object is Tag {
    return (typeof object === 'string' || (object as any) instanceof String)
}


function tag(
    tag: string,
    attrs: Object,
    children: Array<Html>,
    apply?: (element: HTMLElement) => void
): Tag {
    return {tag: tag, attrs: attrs, children: children, apply: apply}
}

function layout_html(html: Html): Node {
    if (instanceOfElement(html)) {
        let node = document.createElement(html.tag);
        for (const child of html.children) {
            node.appendChild(layout_html(child));
        }
        for (const key of Object.keys(html.attrs)) {
            const value = (html.attrs as any)[key];
            if ((typeof value === 'string') || ((value as any) instanceof String)) {
                node.setAttribute(key, value);
            } else {
                node.setAttribute(key, JSON.stringify(value));
            }
        }
        if (html.apply) {
            html.apply(node);
        }
        return node as Node;
    } else if (instanceOfText(html)) {
        return document.createTextNode(html) as Node;
    } else {
        throw "Invalid Node"
    }
}

function uuid(): string {
    return 'UID_xxxxxxxx_xxxx_4xxx_yxxx_xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
        var r = Math.random() * 16 | 0, v = c == 'x' ? r : (r & 0x3 | 0x8);
        return v.toString(16);
    });
}


const rust = import('./pkg/index');

// function root_tick() {
//     window.requestAnimationFrame(root_tick);
// }

window.onload = function() {
    console.log('[js] onload');
    let rust_tick: any = null;

    function go() {
        rust_tick();
        window.requestAnimationFrame(go);
    }
    rust.then((moduels) => {
        console.log('[js] rust modules loaded');
        moduels.init_system();
        rust_tick = moduels.tick;
        go();
    });
};

