use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 640;
const HEIGHT: usize = 480;

fn set_pixel(
    buffer: &mut [u32],
    x: i32,
    y: i32,
    colour: u32,
) {
    if x < 0 || y < 0 {
        return;
    }

    let x = x as usize;
    let y = y as usize;

    if x >= WIDTH || y >= HEIGHT {
        return;
    }

    buffer[y * WIDTH + x] = colour;
}

fn draw_line(
    buffer: &mut [u32],
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    colour: u32,
) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();

    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };

    let mut err = dx + dy;

    let mut x = x0;
    let mut y = y0;

    loop {
        set_pixel(buffer, x, y, colour);

        if x == x1 && y == y1 {
            break;
        }

        let e2 = 2 * err;

        if e2 >= dy {
            err += dy;
            x += sx;
        }

        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

fn edge_function(ax: i32, ay: i32, bx: i32, by: i32, px: i32, py: i32) -> i32 {
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}

fn draw_filled_triangle(
    buffer: &mut [u32],
    v0: (i32, i32),
    v1: (i32, i32),
    v2: (i32, i32),
    colour: u32,
) {
    let (x0, y0) = v0;
    let (x1, y1) = v1;
    let (x2, y2) = v2;

    let min_x = x0.min(x1).min(x2).max(0);
    let max_x = x0.max(x1).max(x2).min(WIDTH as i32 - 1);
    let min_y = y0.min(y1).min(y2).max(0);
    let max_y = y0.max(y1).max(y2).min(HEIGHT as i32 - 1);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let w0 = edge_function(x1, y1, x2, y2, x, y);
            let w1 = edge_function(x2, y2, x0, y0, x, y);
            let w2 = edge_function(x0, y0, x1, y1, x, y);

            if w0 >= 0 && w1 >= 0 && w2 >= 0 {
                set_pixel(buffer, x, y, colour);
            }
        }
    }
}

fn barycentric(
    v0: (i32, i32),
    v1: (i32, i32),
    v2: (i32, i32),
    p: (i32, i32),
) -> (f32, f32, f32) {
    let area = edge_function(v0.0, v0.1, v1.0, v1.1, v2.0, v2.1) as f32;

    let w0 = edge_function(v1.0, v1.1, v2.0, v2.1, p.0, p.1) as f32 / area;
    let w1 = edge_function(v2.0, v2.1, v0.0, v0.1, p.0, p.1) as f32 / area;
    let w2 = edge_function(v0.0, v0.1, v1.0, v1.1, p.0, p.1) as f32 / area;

    (w0, w1, w2)
}

fn unpack_color(c: u32) -> (f32, f32, f32) {
    let r = ((c >> 16) & 0xFF) as f32;
    let g = ((c >> 8) & 0xFF) as f32;
    let b = (c & 0xFF) as f32;
    (r, g, b)
}

fn pack_color(r: f32, g: f32, b: f32) -> u32 {
    let r = r.clamp(0.0, 255.0) as u32;
    let g = g.clamp(0.0, 255.0) as u32;
    let b = b.clamp(0.0, 255.0) as u32;
    (r << 16) | (g << 8) | b
}

fn draw_interpolated_triangle(
    buffer: &mut [u32],
    v0: (i32, i32),
    v1: (i32, i32),
    v2: (i32, i32),
    c0: u32,
    c1: u32,
    c2: u32,
) {
    let min_x = v0.0.min(v1.0).min(v2.0).max(0);
    let max_x = v0.0.max(v1.0).max(v2.0).min(WIDTH as i32 - 1);
    let min_y = v0.1.min(v1.1).min(v2.1).max(0);
    let max_y = v0.1.max(v1.1).max(v2.1).min(HEIGHT as i32 - 1);

    let area = edge_function(v0.0, v0.1, v1.0, v1.1, v2.0, v2.1);

    if area == 0 {
        return;
    }

    let (r0, g0, b0) = unpack_color(c0);
    let (r1, g1, b1) = unpack_color(c1);
    let (r2, g2, b2) = unpack_color(c2);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let w0 = edge_function(v1.0, v1.1, v2.0, v2.1, x, y);
            let w1 = edge_function(v2.0, v2.1, v0.0, v0.1, x, y);
            let w2 = edge_function(v0.0, v0.1, v1.0, v1.1, x, y);

            if (w0 >= 0 && w1 >= 0 && w2 >= 0)
                || (w0 <= 0 && w1 <= 0 && w2 <= 0)
            {
                let alpha = w0 as f32 / area as f32;
                let beta  = w1 as f32 / area as f32;
                let gamma = w2 as f32 / area as f32;

                let r = alpha * r0 + beta * r1 + gamma * r2;
                let g = alpha * g0 + beta * g1 + gamma * g2;
                let b = alpha * b0 + beta * b1 + gamma * b2;

                set_pixel(buffer, x, y, pack_color(r, g, b));
            }
        }
    }
}

fn main() {
    let mut window = Window::new(
        "Software Rasterizer",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
	.expect("Unable to open window");

    // Framebuffer: one u32 per pixel (0x00RRGGBB)
    let mut buffer = vec![0u32; WIDTH * HEIGHT];

    // Fill background (dark grey)
    for pixel in buffer.iter_mut() {
        *pixel = 0x202020;
    }

    set_pixel(&mut buffer, 320, 240, 0xFF0000);
    draw_line(&mut buffer, 100, 100, 500, 300, 0x00FF00);
    draw_line(&mut buffer, 100, 300, 500, 100, 0x00FF00);
    draw_filled_triangle(
	&mut buffer,
	(200, 100),
	(300, 350),
	(400, 150),
	0x0080FF,
    );
    draw_interpolated_triangle(
	&mut buffer,
	(20, 100),
	(300, 350),
	(400, 150),
	0xFF0000, // red
	0x00FF00, // green
	0x0000FF, // blue
    );

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
