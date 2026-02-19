use image::Rgba;
use rand::Rng;
use std::path::Path;

use crate::common::{new_image, save_image, TILE_SIZE};

pub fn generate_cave(output_dir: &Path) {
    let mut img = new_image();
    let mut rng = rand::thread_rng();

    let rock_bg = Rgba([100, 95, 85, 255]);
    let rock_dark = Rgba([70, 65, 55, 255]);
    let rock_mid = Rgba([120, 110, 95, 255]);
    let rock_light = Rgba([150, 140, 120, 255]);
    let cave_black = Rgba([20, 20, 25, 255]);
    let cave_dark = Rgba([40, 40, 45, 255]);
    let moss = Rgba([60, 80, 50, 255]);

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let r: f32 = rng.r#gen();
            let color = if r < 0.2 {
                rock_dark
            } else if r < 0.3 {
                rock_light
            } else {
                rock_bg
            };
            img.put_pixel(x, y, color);
        }
    }

    let cave_center_x = 8;

    let arch_shape = [
        (4, 3), (5, 5), (6, 7), (7, 8), (8, 9), (9, 9),
        (10, 9), (11, 9), (12, 9), (13, 9), (14, 9), (15, 9),
    ];

    for (y, width) in arch_shape.iter() {
        if *y >= TILE_SIZE {
            continue;
        }
        let half_width = *width / 2;
        for dx in 0..=*width {
            let x = cave_center_x - half_width + dx;
            if x >= TILE_SIZE {
                continue;
            }

            let is_edge = dx == 0 || dx == *width;

            let color = if is_edge {
                rock_dark
            } else if *y <= 5 {
                cave_black
            } else if *y <= 8 {
                cave_dark
            } else {
                cave_black
            };

            img.put_pixel(x, *y, color);
        }
    }

    for y in 3..=7 {
        let arch_width = match y {
            3 => 5,
            4 => 7,
            5 => 9,
            6 => 10,
            7 => 11,
            _ => 11,
        };
        let half_width = arch_width / 2;

        for dx in [0, arch_width] {
            let x = cave_center_x - half_width + dx;
            if x < TILE_SIZE && y < TILE_SIZE {
                let color = if dx == 0 {
                    rock_dark
                } else {
                    rock_light
                };
                img.put_pixel(x, y, color);

                if dx == 0 && x > 0 {
                    img.put_pixel(x - 1, y, rock_mid);
                } else if dx == arch_width && x + 1 < TILE_SIZE {
                    img.put_pixel(x + 1, y, rock_light);
                }
            }
        }
    }

    for y in 8..TILE_SIZE {
        for x in 2..=3 {
            if x < TILE_SIZE && y < TILE_SIZE {
                let color = if x == 2 { rock_dark } else { rock_mid };
                img.put_pixel(x, y, color);
            }
        }
        for x in 12..=13 {
            if x < TILE_SIZE && y < TILE_SIZE {
                let color = if x == 13 { rock_light } else { rock_mid };
                img.put_pixel(x, y, color);
            }
        }
    }

    img.put_pixel(5, 2, rock_mid);
    img.put_pixel(6, 2, rock_light);
    img.put_pixel(5, 3, rock_dark);

    img.put_pixel(9, 2, rock_light);
    img.put_pixel(10, 2, rock_mid);
    img.put_pixel(10, 3, rock_light);

    img.put_pixel(7, 1, rock_dark);
    img.put_pixel(8, 1, rock_mid);
    img.put_pixel(9, 1, rock_mid);
    img.put_pixel(7, 2, rock_mid);
    img.put_pixel(8, 2, rock_dark);

    let moss_positions = [
        (4, 8), (4, 9), (4, 10),
        (12, 9), (12, 10), (12, 11),
        (6, 13), (7, 14), (9, 14),
    ];

    for (mx, my) in moss_positions {
        if mx < TILE_SIZE && my < TILE_SIZE
            && rng.gen_bool(0.7)
        {
            img.put_pixel(mx, my, moss);
        }
    }

    for y in 8..13 {
        for dx in [3, 4, 11, 12] {
            if dx < TILE_SIZE && y < TILE_SIZE
                && rng.gen_bool(0.3)
            {
                let color = if dx < 8 { rock_dark } else { rock_light };
                img.put_pixel(dx, y, color);
            }
        }
    }

    save_image(&img, output_dir, "cave.png");
}
