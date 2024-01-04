import './style.css';
import init, { Site2D, TerrainBuilder } from '../pkg/transport.js';

window.onload = async () => {
    await init();
    const node_num = 30000;
    const bound_max = { x: 100.0, y: 100.0 };

    const terrain = new TerrainBuilder()
        .set_bound_max(new Site2D(bound_max.x, bound_max.y))
        .set_node_num(node_num)
        .set_seed(72)
        .build();

    const img_width = 500;
    const img_height = 500;
    let image_buf = new Uint8ClampedArray(img_width * img_height * 4);
    for (let imgx = 0; imgx < img_width; imgx++) {
        for (let imgy = 0; imgy < img_height; imgy++) {
            const x = bound_max.x * (imgx / img_width);
            const y = bound_max.y * (imgy / img_height);
            const site = new Site2D(x, y);
            const altitude = terrain.get_altitude(site);
            if (altitude) {
                const color = get_color(altitude);
                const index = (imgx + imgy * img_width) * 4;
                image_buf[index] = color[0];
                image_buf[index + 1] = color[1];
                image_buf[index + 2] = color[2];
                image_buf[index + 3] = 255;
            }
        }
    }

    let canvas = document.getElementById('canvasMain') as HTMLCanvasElement;
    canvas.width = img_width;
    canvas.height = img_height;
    let ctx = canvas.getContext('2d') as CanvasRenderingContext2D;
    let imageData = ctx.createImageData(img_width, img_height);
    imageData.data.set(image_buf);
    ctx.putImageData(imageData, 0, 0);
};

const color_table: [[number, number, number], number][] = [
    [[70, 150, 200], 0.0],
    [[240, 240, 210], 0.1],
    [[190, 200, 120], 0.3],
    [[25, 100, 25], 6.0],
    [[15, 60, 15], 8.0],
    [[255, 255, 255], 15.0],
];

const get_color = (altitude: number) => {
    const color_index = (() => {
        let i = 0;
        while (i < color_table.length) {
            if (altitude < color_table[i][1]) {
                break;
            }
            i += 1;
        }
        return i;
    })();

    if (color_index == 0) {
        return color_table[0][0];
    } else if (color_index == color_table.length) {
        return color_table[color_table.length - 1][0];
    } else {
        const color_a = color_table[color_index - 1];
        const color_b = color_table[color_index];

        const prop_a = color_a[1];
        const prop_b = color_b[1];

        const prop = (altitude - prop_a) / (prop_b - prop_a);

        return [
            color_a[0][0] + (color_b[0][0] - color_a[0][0]) * prop,
            color_a[0][1] + (color_b[0][1] - color_a[0][1]) * prop,
            color_a[0][2] + (color_b[0][2] - color_a[0][2]) * prop,
        ];
    }
};