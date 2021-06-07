// import init from 'mathjax-full';

///////////////////////////////////////////////////////////////////////////////
// REGISTER
///////////////////////////////////////////////////////////////////////////////



///////////////////////////////////////////////////////////////////////////////
// ENTRYPOINT
///////////////////////////////////////////////////////////////////////////////

type CanvasInstance = {
    canvas: HTMLCanvasElement;
    ctx: CanvasRenderingContext2D;
};


function setup_canvas({canvas, ctx}: CanvasInstance) {
    // Get the device pixel ratio, falling back to 1.
    var dpr = window.devicePixelRatio || 1;
    // Get the size of the canvas in CSS pixels.
    var rect = canvas.getBoundingClientRect();
    // Give the canvas pixel dimensions of their CSS
    // size * the device pixel ratio.
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    // Scale all drawing operations by the dpr, so you
    // don't have to worry about the difference.
    // ctx.scale(dpr, dpr);
    // console.log(`${canvas.width}x${canvas.height}`);
}


type Point = {
    x: number;
    y: number;
};


function draw_line(ctx: CanvasRenderingContext2D, start: Point, end: Point) {
    ctx.beginPath();
    ctx.moveTo(start.x, start.y);
    ctx.lineTo(end.x, end.y);
    ctx.stroke();
}

function draw_point(
    ctx: CanvasRenderingContext2D,
    point: {location: Point, color?: string, radius?: number}
) {
    let radius = point.radius ? point.radius : 10;
    ctx.beginPath();
    ctx.arc(point.location.x, point.location.y, radius, 0, 2 * Math.PI);
    if (point.color) {
        ctx.fillStyle = point.color;
    }
    ctx.closePath();
    ctx.fill();
}

function draw({canvas, ctx}: CanvasInstance) {
    const width = canvas.width;
    const height = canvas.height;

    draw_point(ctx, {
        location: {
            x: width * 0.5,
            y: height * 0.5,
        },
        color: "#999"
    });
}


function start() {
    ///////////////////////////////////////////////////////////////////////////
    // SETUP
    ///////////////////////////////////////////////////////////////////////////
    const canvas = document.createElement('canvas');
    document.body.appendChild(canvas);
    let mouse_state = {
        down: false
    };
    let canvas_instance = {
        canvas,
        ctx: canvas.getContext('2d'),
    };
    ///////////////////////////////////////////////////////////////////////////
    // MOUSE EVENTS
    ///////////////////////////////////////////////////////////////////////////
    type FatPoint = {
        point: Point,
        drawn: boolean,
    };
    type Segment = {
        points: Array<FatPoint>,
    };
    let segments: Array<Segment> = [];
    let start_new_segment = true;
    const register_point = (point: FatPoint) => {
        if (start_new_segment) {
            segments.push({
                points: [point],
            });
            start_new_segment = false;
        } else {
            segments[segments.length - 1].points.push(point);
        }
    };
    const get_current_point = (event: MouseEvent): FatPoint => {
        const dpr = window.devicePixelRatio || 1;
        const rect = canvas.getBoundingClientRect();
        const x = Math.round((event.clientX - rect.left) * dpr);
        const y = Math.round((event.clientY - rect.top) * dpr);
        return {point: {x, y}, drawn: false}
    };
    canvas.addEventListener('mousedown', (event) => {
        mouse_state.down = true;
        start_new_segment = true;
        register_point(get_current_point(event));
    });
    canvas.addEventListener('mouseup', (event) => {
        mouse_state.down = false;
        register_point(get_current_point(event));
    });
    canvas.addEventListener("mousemove", (event) => {
        if (mouse_state.down) {
            register_point(get_current_point(event));
        }
    });
    ///////////////////////////////////////////////////////////////////////////
    // TICK
    ///////////////////////////////////////////////////////////////////////////
    let need_refresh = true;
    const refresh = () => {
        canvas_instance.ctx = canvas.getContext('2d');
        setup_canvas(canvas_instance);
    };
    window.onresize = () => {
        need_refresh = true;
    };
    window.onkeypress = (event: KeyboardEvent) => {
        if (event.key === 'Enter') {
            need_refresh = true;
        }
    };
    const distance_between = (a: Point, b: Point): number => {
        console.assert(a != null);
        console.assert(b != null);
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        return Math.sqrt(Math.pow(dx, 2) + Math.pow(dy, 2));
    };
    const direction_from = (a: Point, b: Point): number => {
        console.assert(a != null);
        console.assert(b != null);
        const num = b.y - a.y;
        const den = b.x - a.x;
        return Math.atan(num / den);
    };
    const green_color = 'rgb(4 251 49 / 63%)';
    const red_color = 'rgb(251 4 4 / 63%)';
    const blue_color = 'rgb(4 193 251 / 63%)';
    const default_color = 'hsl(0deg 0% 69% / 77%)';
    const render = () => {
        let ctx = canvas_instance.ctx;
        segments.forEach((segment, six) => {
            let last_point: Point | null = null;
            segment.points.forEach((fat_point, pix) => {
                let {point} = fat_point;
                let first_point = pix == 0;
                if (fat_point.drawn) {
                    last_point = point;
                    return;
                }
                if (first_point) {
                    ctx.moveTo(point.x, point.y)
                    fat_point.drawn = true;
                    last_point = point;
                    return;
                }
                // DRAW
                let distance = distance_between(point, last_point);
                draw_line(ctx, last_point, point);
                // DONE
                fat_point.drawn = true;
                last_point = point;
            });
        });
        // console.log(`segments: ${segments.length}`);
    };
    const tick = () => {
        if (need_refresh) {
            refresh();
            need_refresh = false;
        }
        render();
        window.requestAnimationFrame(tick);
    };
    window.requestAnimationFrame(tick);
}

window.onload = () => {
    start();
}; 

export {}
