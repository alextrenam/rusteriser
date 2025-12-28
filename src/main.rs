use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 640;
const HEIGHT: usize = 480;

struct Vec2 {
    x: f32,
    y: f32,
}

struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Copy, Clone)]
struct IntVec2 {
    x: i32,
    y: i32,
}

impl IntVec2 {
    pub fn from(vec: &Vec2) -> Self {
	Self {
	    x: vec.x.round() as i32,
	    y: vec.y.round() as i32,
	}
    }
}

struct Fragment {
    position: IntVec2,
    depth: f32,
    colour: u32,
}

fn write_fragment(
    colour_buffer: &mut [u32],
    depth_buffer: &mut [f32],
    fragment: Fragment,
) {
    if fragment.position.x < 0 || fragment.position.y < 0 {
        return;
    }

    let x = fragment.position.x as usize;
    let y = fragment.position.y as usize;

    if x >= WIDTH || y >= HEIGHT {
        return;
    }

    let buffer_index = y * WIDTH + x;
    if fragment.depth < depth_buffer[buffer_index] {
	depth_buffer[buffer_index] = fragment.depth;
	colour_buffer[buffer_index] = fragment.colour;
    }
}

fn draw_line(
    colour_buffer: &mut [u32],
    depth_buffer: &mut [f32],
    v0: IntVec2,
    v1: IntVec2,
    depth: f32,
    colour: u32,
) {
    let dx = (v1.x - v0.x).abs();
    let dy = -(v1.y - v0.y).abs();

    // Parameterise the path
    let sx = if v0.x < v1.x { 1 } else { -1 };
    let sy = if v0.y < v1.y { 1 } else { -1 };

    let mut err = dx + dy;
    let mut v = IntVec2{x: v0.x, y: v0.y};

    loop {
	let fragment = Fragment{position: v, depth: depth, colour: colour};
        write_fragment(colour_buffer, depth_buffer, fragment);

        if v.x == v1.x && v.y == v1.y {
            break;
        }

        let e2 = 2 * err;

        if e2 >= dy {
            err += dy;
            v.x += sx;
        }

        if e2 <= dx {
            err += dx;
            v.y += sy;
        }
    }
}

// fn edge_function(ax: i32, ay: i32, bx: i32, by: i32, px: i32, py: i32) -> i32 {
//     (px - ax) * (by - ay) - (py - ay) * (bx - ax)
// }

// fn draw_filled_triangle(
//     buffer: &mut [u32],
//     v0: (i32, i32),
//     v1: (i32, i32),
//     v2: (i32, i32),
//     colour: u32,
// ) {
//     let (x0, y0) = v0;
//     let (x1, y1) = v1;
//     let (x2, y2) = v2;

//     let min_x = x0.min(x1).min(x2).max(0);
//     let max_x = x0.max(x1).max(x2).min(WIDTH as i32 - 1);
//     let min_y = y0.min(y1).min(y2).max(0);
//     let max_y = y0.max(y1).max(y2).min(HEIGHT as i32 - 1);

//     for y in min_y..=max_y {
//         for x in min_x..=max_x {
//             let w0 = edge_function(x1, y1, x2, y2, x, y);
//             let w1 = edge_function(x2, y2, x0, y0, x, y);
//             let w2 = edge_function(x0, y0, x1, y1, x, y);

//             if w0 >= 0 && w1 >= 0 && w2 >= 0 {
//                 set_pixel(buffer, x, y, colour);
//             }
//         }
//     }
// }

// fn barycentric(
//     v0: (i32, i32),
//     v1: (i32, i32),
//     v2: (i32, i32),
//     p: (i32, i32),
// ) -> (f32, f32, f32) {
//     let area = edge_function(v0.0, v0.1, v1.0, v1.1, v2.0, v2.1) as f32;

//     let w0 = edge_function(v1.0, v1.1, v2.0, v2.1, p.0, p.1) as f32 / area;
//     let w1 = edge_function(v2.0, v2.1, v0.0, v0.1, p.0, p.1) as f32 / area;
//     let w2 = edge_function(v0.0, v0.1, v1.0, v1.1, p.0, p.1) as f32 / area;

//     (w0, w1, w2)
// }

// fn unpack_color(c: u32) -> (f32, f32, f32) {
//     let r = ((c >> 16) & 0xFF) as f32;
//     let g = ((c >> 8) & 0xFF) as f32;
//     let b = (c & 0xFF) as f32;
//     (r, g, b)
// }

// fn pack_color(r: f32, g: f32, b: f32) -> u32 {
//     let r = r.clamp(0.0, 255.0) as u32;
//     let g = g.clamp(0.0, 255.0) as u32;
//     let b = b.clamp(0.0, 255.0) as u32;
//     (r << 16) | (g << 8) | b
// }

// fn draw_interpolated_triangle(
//     buffer: &mut [u32],
//     v0: (i32, i32),
//     v1: (i32, i32),
//     v2: (i32, i32),
//     c0: u32,
//     c1: u32,
//     c2: u32,
// ) {
//     let min_x = v0.0.min(v1.0).min(v2.0).max(0);
//     let max_x = v0.0.max(v1.0).max(v2.0).min(WIDTH as i32 - 1);
//     let min_y = v0.1.min(v1.1).min(v2.1).max(0);
//     let max_y = v0.1.max(v1.1).max(v2.1).min(HEIGHT as i32 - 1);

//     let area = edge_function(v0.0, v0.1, v1.0, v1.1, v2.0, v2.1);

//     if area == 0 {
//         return;
//     }

//     let (r0, g0, b0) = unpack_color(c0);
//     let (r1, g1, b1) = unpack_color(c1);
//     let (r2, g2, b2) = unpack_color(c2);

//     for y in min_y..=max_y {
//         for x in min_x..=max_x {
//             let w0 = edge_function(v1.0, v1.1, v2.0, v2.1, x, y);
//             let w1 = edge_function(v2.0, v2.1, v0.0, v0.1, x, y);
//             let w2 = edge_function(v0.0, v0.1, v1.0, v1.1, x, y);

//             if (w0 >= 0 && w1 >= 0 && w2 >= 0)
//                 || (w0 <= 0 && w1 <= 0 && w2 <= 0)
//             {
//                 let alpha = w0 as f32 / area as f32;
//                 let beta  = w1 as f32 / area as f32;
//                 let gamma = w2 as f32 / area as f32;

//                 let r = alpha * r0 + beta * r1 + gamma * r2;
//                 let g = alpha * g0 + beta * g1 + gamma * g2;
//                 let b = alpha * b0 + beta * b1 + gamma * b2;

//                 set_pixel(buffer, x, y, pack_color(r, g, b));
//             }
//         }
//     }
// }

fn main() {
    let mut window = Window::new(
        "Software Rasterizer",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
	.expect("Unable to open window");

    // Buffers for storing pixel depths and colours
    let mut colour_buffer = vec![0u32; WIDTH * HEIGHT];
    let mut depth_buffer = vec![f32::INFINITY; WIDTH * HEIGHT];
    
    // Fill background (dark grey)
    for pixel in colour_buffer.iter_mut() {
        *pixel = 0x202020;
    }

    let point = Vec2{x: 320.0, y: 240.0};
    let pixel_position = IntVec2::from(&point);
    let fragment = Fragment{position:pixel_position, depth: 0.0, colour: 0xFF0000};
    write_fragment(&mut colour_buffer, &mut depth_buffer, fragment);
    
    let pixel_1 = IntVec2{x: 100, y: 100};
    let pixel_2 = IntVec2{x: 500, y: 300};
    let pixel_3 = IntVec2{x: 100, y: 300};
    let pixel_4 = IntVec2{x: 500, y: 100};
    let colour = 0x00FF00;
    
    draw_line(&mut colour_buffer, &mut depth_buffer, pixel_1, pixel_2, 0.0, colour);
    draw_line(&mut colour_buffer, &mut depth_buffer, pixel_3, pixel_4, 0.0, colour);
    
    // draw_filled_triangle(
    // 	&mut buffer,
    // 	(200, 100),
    // 	(300, 350),
    // 	(400, 150),
    // 	0x0080FF,
    // );
    // draw_interpolated_triangle(
    // 	&mut buffer,
    // 	(20, 100),
    // 	(300, 350),
    // 	(400, 150),
    // 	0xFF0000, // red
    // 	0x00FF00, // green
    // 	0x0000FF, // blue
    // );

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&colour_buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
