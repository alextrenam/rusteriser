// Window handling
use minifb::{Key, Window, WindowOptions};

// Linear algebra
mod vector;
use vector::{Vec2, Vec3, IntVec2};

// // Rasterising
// mod raster;
// use raster::{};


const WIDTH: usize = 640;
const HEIGHT: usize = 480;

// 3D geometry
struct Vertex {
    position: Vec3,
    colour: u32,
}

// Screen space geometry
struct ScreenVertex {
    position: IntVec2,
    depth: f32,
    colour: u32,
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
    z0: f32,
    v1: IntVec2,
    z1: f32,
    colour: u32,
) {
    let dx = (v1.x - v0.x).abs();
    let dy = (v1.y - v0.y).abs();

    let sx = if v0.x < v1.x { 1 } else { -1 };
    let sy = if v0.y < v1.y { 1 } else { -1 };

    let mut err = dx - dy;

    let steps = dx.max(dy).max(1) as f32;
    let mut t = 0.0;
    let dt = 1.0 / steps;

    let mut v = v0;

    loop {
        let depth = (1.0 - t) * z0 + t * z1;

        let fragment = Fragment {
            position: v,
            depth,
            colour,
        };
        write_fragment(colour_buffer, depth_buffer, fragment);

        if v.x == v1.x && v.y == v1.y {
            break;
        }

        let e2 = 2 * err;

        if e2 > -dy {
            err -= dy;
            v.x += sx;
        }

        if e2 < dx {
            err += dx;
            v.y += sy;
        }

        t += dt;
    }
}

fn twice_signed_area(
    v0: IntVec2,
    v1: IntVec2,
    v2: IntVec2,
) -> i32 {
    // Positive => vertices ordered counterclockwise
    // Negative => vertices ordered clockwise
    // Zero => degenerate (colinear)
    (v2.x - v0.x) * (v1.y - v0.y) - (v2.y - v0.y) * (v1.x - v0.x)
}

fn barycentric(
    v0: IntVec2,
    v1: IntVec2,
    v2: IntVec2,
    position: IntVec2,
) -> (f32, f32, f32) {
    let area = twice_signed_area(v0, v1, v2) as f32;

    let weight0 = twice_signed_area(v1, v2, position) as f32 / area;
    let weight1 = twice_signed_area(v2, v0, position) as f32 / area;
    let weight2 = twice_signed_area(v0, v1, position) as f32 / area;

    (weight0, weight1, weight2)
}

fn unpack_colour(colour: u32) -> (f32, f32, f32) {
    let red = ((colour >> 16) & 0xFF) as f32;
    let green = ((colour >> 8) & 0xFF) as f32;
    let blue = (colour & 0xFF) as f32;
    (red, green, blue)
}

fn pack_colour(red: f32, green: f32, blue: f32) -> u32 {
    let red = red.clamp(0.0, 255.0) as u32;
    let green = green.clamp(0.0, 255.0) as u32;
    let blue = blue.clamp(0.0, 255.0) as u32;
    (red << 16) | (green << 8) | blue
}

fn draw_triangle(
    colour_buffer: &mut [u32],
    depth_buffer: &mut [f32],
    screen_vertex0: ScreenVertex,
    screen_vertex1: ScreenVertex,
    screen_vertex2: ScreenVertex,
) {
    let min_x =
	screen_vertex0.position.x
	.min(screen_vertex1.position.x)
	.min(screen_vertex2.position.x)
	.max(0);
    let max_x =
	screen_vertex0.position.x
	.max(screen_vertex1.position.x)
	.max(screen_vertex2.position.x)
	.min(WIDTH as i32 - 1);
    let min_y =
	screen_vertex0.position.y
	.min(screen_vertex1.position.y)
	.min(screen_vertex2.position.y)
	.max(0);
    let max_y =
	screen_vertex0.position.y
	.max(screen_vertex1.position.y)
	.max(screen_vertex2.position.y)
	.min(HEIGHT as i32 - 1);

    let area = twice_signed_area(
	screen_vertex0.position,
	screen_vertex1.position,
	screen_vertex2.position);

    if area == 0 {
        return;
    }

    let inv_z0 = 1.0 / screen_vertex0.depth;
    let inv_z1 = 1.0 / screen_vertex1.depth;
    let inv_z2 = 1.0 / screen_vertex2.depth;

    let (red0, green0, blue0) = unpack_colour(screen_vertex0.colour);
    let (red1, green1, blue1) = unpack_colour(screen_vertex1.colour);
    let (red2, green2, blue2) = unpack_colour(screen_vertex2.colour);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
	    let position = IntVec2{x, y};
	    
	    let (weight0, weight1, weight2) =
		barycentric(screen_vertex0.position,
			    screen_vertex1.position,
			    screen_vertex2.position,
			    position);
	    
	    if (weight0 >= 0.0 && weight1 >= 0.0 && weight2 >= 0.0)
		|| (weight0 <= 0.0 && weight1 <= 0.0 && weight2 <= 0.0) {
		    let inv_z = weight0 * inv_z0 + weight1 * inv_z1 + weight2 * inv_z2;
		    let depth = 1.0 / inv_z;

		    let colour =
			if screen_vertex0.colour == screen_vertex1.colour
			&& screen_vertex1.colour == screen_vertex2.colour {
			screen_vertex0.colour
		    } else {
			let red = (
			    weight0 * red0 * inv_z0
				+ weight1 * red1 * inv_z1
				+ weight2 * red2 * inv_z2
			) / inv_z;
			let green = (
			    weight0 * green0 * inv_z0
				+ weight1 * green1 * inv_z1
				+ weight2 * green2 * inv_z2
			) / inv_z;
			let blue = (
			    weight0 * blue0 * inv_z0
				+ weight1 * blue1 * inv_z1
				+ weight2 * blue2 * inv_z2
			) / inv_z;
			pack_colour(red, green, blue)
		    };
		    
		    let fragment = Fragment{position, depth, colour};
                    write_fragment(colour_buffer, depth_buffer, fragment);
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
    
    draw_line(
	&mut colour_buffer,
	&mut depth_buffer,
	pixel_1,
	0.1,
	pixel_2,
	0.5,
	colour);
    draw_line(
	&mut colour_buffer,
	&mut depth_buffer,
	pixel_3,
	0.0,
	pixel_4,
	0.5,
	colour);

    let pixel_5 = IntVec2{x: 200, y: 100};
    let pixel_6 = IntVec2{x: 300, y: 350};
    let pixel_7 = IntVec2{x: 400, y: 150};
    let colour_2 = 0x00CC0;

    let screen_vertex0 = ScreenVertex{position: pixel_5, depth: 2.0, colour: colour_2};
    let screen_vertex1 = ScreenVertex{position: pixel_6, depth: 2.0, colour: colour_2};
    let screen_vertex2 = ScreenVertex{position: pixel_7, depth: 2.0, colour: colour_2};
    
    // draw_triangle(
    // 	&mut colour_buffer,
    // 	&mut depth_buffer,
    // 	screen_vertex0,
    // 	screen_vertex1,
    // 	screen_vertex2,
    // );
    
    let screen_vertex3 = ScreenVertex{position: pixel_5, depth: 1.0, colour: 0xFF0000};  // Red
    let screen_vertex4 = ScreenVertex{position: pixel_6, depth: 1.0, colour: 0x00FF00};  // Green
    let screen_vertex5 = ScreenVertex{position: pixel_7, depth: 2.0, colour: 0x0000FF};  // Blue
    
    draw_triangle(
	&mut colour_buffer,
	&mut depth_buffer,
	screen_vertex3,
	screen_vertex4,
	screen_vertex5,
    );
    
    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&colour_buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
