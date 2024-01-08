import './style.css';
import init, {
    TerrainBuilder,
    TransportNetworkBuilder,
} from '../pkg/transport.js';

window.onload = async () => {
    await init();
    const node_num = 20000;
    const bound_max = { x: 200.0, y: 100.0 };

    const terrain = new TerrainBuilder()
        .set_bound_max(bound_max.x, bound_max.y)
        .set_node_num(node_num)
        .build(100);

    const img_width = 1000;
    const img_height = 500;
    let image_buf = new Uint8ClampedArray(img_width * img_height * 4);
    for (let imgx = 0; imgx < img_width; imgx++) {
        for (let imgy = 0; imgy < img_height; imgy++) {
            const x = bound_max.x * (imgx / img_width);
            const y = bound_max.y * (imgy / img_height);
            const altitude = terrain.get_altitude(x, y);
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

    const transport = new TransportNetworkBuilder()
        .set_start(bound_max.x / 2.0, bound_max.y / 2.0)
        .set_iterations(34000)
        .set_branch_angle_deviation(Math.PI / 40.0)
        .set_branch_length(0.5)
        .set_normal_rotation_probability(0.8)
        .set_highway_rotation_probability(0.02)
        .set_highway_construction_priority(30.0)
        .set_even_path_length_weight(1.5)
        .set_highway_path_length_weight(1.5)
        .set_branch_max_angle(Math.PI / 40.0)
        .build(0, terrain);

    let canvas = document.getElementById('canvasMain') as HTMLCanvasElement;
    canvas.width = img_width;
    canvas.height = img_height;
    let ctx = canvas.getContext('2d') as CanvasRenderingContext2D;
    let imageData = ctx.createImageData(img_width, img_height);
    imageData.data.set(image_buf);
    ctx.putImageData(imageData, 0, 0);

    // white
    ctx.fillStyle = 'rgba(255, 255, 255, 0.7)';
    ctx.fillRect(0, 0, img_width, img_height);

    for (let i = 0; i < transport.num_nodes(); i++) {
        const site = transport.get_site(i);
        const neighbors = transport.get_neighbors(i);
        for (let j = 0; j < neighbors.length; j++) {
            const neighbor = transport.get_site(neighbors[j].index);
            const [sx, sy] = [
                (site.x / bound_max.x) * img_width,
                (site.y / bound_max.y) * img_height,
            ];
            const [ex, ey] = [
                (neighbor.x / bound_max.x) * img_width,
                (neighbor.y / bound_max.y) * img_height,
            ];
            let lineWidth = 0.0;
            if (neighbors[j].is_highway) {
                lineWidth = 2;
            } else {
                lineWidth = 0.5;
            }
            ctx.beginPath();
            ctx.moveTo(sx, sy);
            ctx.lineTo(ex, ey);

            ctx.lineWidth = lineWidth;
            ctx.strokeStyle = 'rgba(50, 50, 50, 0.7)';
            ctx.stroke();
        }
    }
};

const color_table: [[number, number, number], number][] = [
    [[70, 150, 200], 0.0],
    [[240, 240, 210], 0.1],
    [[190, 200, 120], 0.3],
    [[170, 180, 100], 2.0],
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
