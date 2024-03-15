#![allow(unused, dead_code)]

#[derive(Debug, Clone)]
pub struct Bitmap {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl Bitmap {
    pub fn new(width: u32, height: u32) -> Self {
        let data = vec![255u8; (width * height * 4) as usize];
        Self {
            width,
            height,
            data,
        }
    }

    pub fn set_pixel(&mut self, x: i64, y: i64, r: u8, g: u8, b: u8, a: u8) {
        if x >= 0 && y < (self.width as i64) && y >= 0 && y < (self.height as i64) {
            let x = x as usize;
            let y = y as usize;
            let idx = 4 * (y * (self.width as usize) + x);

            if (idx + 3) < self.data.len() {
                self.data[idx] = r;
                self.data[idx + 1] = g;
                self.data[idx + 2] = b;
                self.data[idx + 3] = a;
            }
        }
    }

    pub fn draw_line(&mut self, from: (i64, i64), to: (i64, i64), r: u8, g: u8, b: u8) {
        fn round(x: f64, dir: i64) -> i64 {
            if x >= 0.0 {
                let i = x as i64;
                let frac = x.fract();

                if dir > 0 {
                    if frac >= 0.5 {
                        i + 1
                    } else {
                        i
                    }
                } else if frac > 0.5 {
                    i + 1
                } else {
                    i
                }
            } else {
                let i = x as i64;
                let frac = x.fract();
                if dir > 0 {
                    if frac > 0.5 {
                        i - 1
                    } else {
                        i
                    }
                } else if frac >= 0.5 {
                    i - 1
                } else {
                    i
                }
            }
        }

        struct Point {
            x: i64,
            y: i64,
        };

        impl Point {
            fn new(x: i64, y: i64) -> Self {
                Self { x, y }
            }
        }

        let p0 = Point::new(from.0, from.1);
        let p1 = Point::new(to.0, to.1);

        let dx = p1.x - p0.x;
        let dy = p1.y - p0.y;
        let mut x = p0.x as f64;
        let mut y = p0.y as f64;
        let (mut n, i, j) = if dy.abs() > dx.abs() {
            let j = if dy < 0 { -1.0 } else { 1.0 };
            let i = j * ((dx as f64) / (dy as f64));
            (dy.abs() + 1, i, j)
        } else {
            let i = if dx < 0 { -1.0 } else { 1.0 };
            let j = i * (dy as f64) / (dx as f64);
            (dx.abs() + 1, i, j)
        };

        while n > 0 {
            self.set_pixel(round(x, dx), round(y, dy), r, g, b, 255);
            x += i;
            y += j;
            n -= 1;
        }
    }

    pub fn write_texture(&self, queue: &wgpu::Queue, texture: &wgpu::Texture) {
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            &self.data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.width),
                rows_per_image: Some(self.height),
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
    }
}
