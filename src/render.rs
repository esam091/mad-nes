use std::{convert::TryInto, ops::BitAnd};

use sdl2::{
    pixels::{Color, Palette, PixelFormatEnum},
    render::Texture,
    surface::Surface,
    video::Window,
};
use sdl2::{
    render::{Canvas, TextureCreator},
    video::WindowContext,
};

use crate::ppu::Ppu;

const PALETTE: [(u8, u8, u8); 64] = [
    (0x80, 0x80, 0x80),
    (0x00, 0x3D, 0xA6),
    (0x00, 0x12, 0xB0),
    (0x44, 0x00, 0x96),
    (0xA1, 0x00, 0x5E),
    (0xC7, 0x00, 0x28),
    (0xBA, 0x06, 0x00),
    (0x8C, 0x17, 0x00),
    (0x5C, 0x2F, 0x00),
    (0x10, 0x45, 0x00),
    (0x05, 0x4A, 0x00),
    (0x00, 0x47, 0x2E),
    (0x00, 0x41, 0x66),
    (0x00, 0x00, 0x00),
    (0x05, 0x05, 0x05),
    (0x05, 0x05, 0x05),
    (0xC7, 0xC7, 0xC7),
    (0x00, 0x77, 0xFF),
    (0x21, 0x55, 0xFF),
    (0x82, 0x37, 0xFA),
    (0xEB, 0x2F, 0xB5),
    (0xFF, 0x29, 0x50),
    (0xFF, 0x22, 0x00),
    (0xD6, 0x32, 0x00),
    (0xC4, 0x62, 0x00),
    (0x35, 0x80, 0x00),
    (0x05, 0x8F, 0x00),
    (0x00, 0x8A, 0x55),
    (0x00, 0x99, 0xCC),
    (0x21, 0x21, 0x21),
    (0x09, 0x09, 0x09),
    (0x09, 0x09, 0x09),
    (0xFF, 0xFF, 0xFF),
    (0x0F, 0xD7, 0xFF),
    (0x69, 0xA2, 0xFF),
    (0xD4, 0x80, 0xFF),
    (0xFF, 0x45, 0xF3),
    (0xFF, 0x61, 0x8B),
    (0xFF, 0x88, 0x33),
    (0xFF, 0x9C, 0x12),
    (0xFA, 0xBC, 0x20),
    (0x9F, 0xE3, 0x0E),
    (0x2B, 0xF0, 0x35),
    (0x0C, 0xF0, 0xA4),
    (0x05, 0xFB, 0xFF),
    (0x5E, 0x5E, 0x5E),
    (0x0D, 0x0D, 0x0D),
    (0x0D, 0x0D, 0x0D),
    (0xFF, 0xFF, 0xFF),
    (0xA6, 0xFC, 0xFF),
    (0xB3, 0xEC, 0xFF),
    (0xDA, 0xAB, 0xEB),
    (0xFF, 0xA8, 0xF9),
    (0xFF, 0xAB, 0xB3),
    (0xFF, 0xD2, 0xB0),
    (0xFF, 0xEF, 0xA6),
    (0xFF, 0xF7, 0x9C),
    (0xD7, 0xE8, 0x95),
    (0xA6, 0xED, 0xAF),
    (0xA2, 0xF2, 0xDA),
    (0x99, 0xFF, 0xFC),
    (0xDD, 0xDD, 0xDD),
    (0x11, 0x11, 0x11),
    (0x11, 0x11, 0x11),
];

fn palette_number(left: u8, right: u8, index: usize) -> u8 {
    let is_left_on = left.bitand(1 << (7 - index)) != 0;
    let is_right_on = right.bitand(1 << (7 - index)) != 0;

    match (is_left_on, is_right_on) {
        (false, false) => 0,
        (true, false) => 1,
        (false, true) => 2,
        (true, true) => 3,
    }
}

fn create_debug_texture<T>(
    texture_creator: &TextureCreator<T>,
    pixel_format: PixelFormatEnum,
) -> Texture<'_> {
    let mut surface = Surface::new(256, 240, pixel_format).unwrap();

    // tile grid
    let color = sdl2::pixels::Color::RGBA(0x80, 0x80, 0x80, 0x80);

    for row in (8..240).step_by(8) {
        surface
            .fill_rect(sdl2::rect::Rect::new(0, row, 256, 1), color)
            .unwrap();
    }

    for col in (8..256).step_by(8) {
        surface
            .fill_rect(sdl2::rect::Rect::new(col, 0, 1, 240), color)
            .unwrap();
    }

    // attribute grid
    let color = sdl2::pixels::Color::RGBA(0xff, 0xff, 0xff, 0x90);

    for row in (16..240).step_by(16) {
        surface
            .fill_rect(sdl2::rect::Rect::new(0, row, 256, 1), color)
            .unwrap();
    }

    for col in (16..256).step_by(16) {
        surface
            .fill_rect(sdl2::rect::Rect::new(col, 0, 1, 240), color)
            .unwrap();
    }

    texture_creator
        .create_texture_from_surface(surface)
        .unwrap()
}

const SCALE: u32 = 2;

pub struct Renderer<'a> {
    canvas: Canvas<Window>,

    texture_creator: &'a TextureCreator<WindowContext>,
    debug_texture: Texture<'a>,
    gameplay_texture: Texture<'a>,
    left_pattern_texture: Texture<'a>,
}

impl<'a> Renderer<'a> {
    pub fn new(
        canvas: Canvas<Window>,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> Renderer<'a> {
        let debug_texture = create_debug_texture(&texture_creator, canvas.default_pixel_format());

        let gameplay_texture = texture_creator
            .create_texture_target(None, 256, 240)
            .unwrap();

        let left_pattern_texture = texture_creator
            .create_texture_target(None, 128, 128)
            .unwrap();

        Renderer {
            canvas,
            texture_creator,
            debug_texture,
            gameplay_texture,
            left_pattern_texture,
        }
    }

    pub fn render(&mut self, ppu: &Ppu) {
        self.canvas.set_draw_color(Color::GRAY);
        self.canvas.clear();

        let video_buffer = ppu.get_buffer();

        let color_set = (0..16)
            .map(|index| {
                let palette_index = if index % 4 == 0 {
                    video_buffer[0x3f00]
                } else {
                    video_buffer[0x3f00 + index]
                };

                let (r, g, b) = PALETTE[palette_index as usize];

                Color::RGB(r, g, b)
            })
            .collect::<Vec<_>>();

        let palette_set = Palette::with_colors(&color_set).unwrap();

        /*
        Generate the pattern tiles.
        The tiles are arranged vertically and also grouped in sets.
        (0, 0) -> (7, 7): Tile 0 palette set 1
        (0, 8) -> (7, 15): Tile 0 pallette set 1
        (0, 2048) -> (7, 2055): Tile 0 pallette set 2
        */
        let mut pattern_surface = Surface::new(8, 256 * 8 * 4, PixelFormatEnum::Index8).unwrap();
        pattern_surface.set_palette(&palette_set).unwrap();
        let pattern_surface_raw = pattern_surface.without_lock_mut().unwrap();

        let pattern_table = ppu.left_pattern_table();
        for index in 0..256 {
            let address = index * 0x10;

            for row in 0..8 {
                let left_bits = pattern_table[address + row];
                let right_bits = pattern_table[address + row + 8];

                for col in 0..8 {
                    let palette_value = palette_number(left_bits, right_bits, col);

                    // fill each palette set
                    pattern_surface_raw[row * 8 + col + index * 64] = palette_value;
                    pattern_surface_raw[row * 8 + col + index * 64 + 2048 * 8] = palette_value + 4;
                    pattern_surface_raw[row * 8 + col + index * 64 + 4096 * 8] = palette_value + 8;
                    pattern_surface_raw[row * 8 + col + index * 64 + 6144 * 8] = palette_value + 12;
                }
            }
        }

        let pattern_texture = self
            .texture_creator
            .create_texture_from_surface(pattern_surface)
            .unwrap();

        self.canvas
            .with_texture_canvas(&mut self.left_pattern_texture, |canvas| {
                for row in 0..16 {
                    for col in 0..16 {
                        let tile_number = row * 16 + col;

                        canvas
                            .copy(
                                &pattern_texture,
                                sdl2::rect::Rect::new(0, tile_number * 8, 8, 8),
                                sdl2::rect::Rect::new(col * 8, row * 8, 8, 8),
                            )
                            .unwrap();
                    }
                }
            })
            .unwrap();

        let debug_texture = &self.debug_texture;
        let gameplay_texture = &mut self.gameplay_texture;
        self.canvas
            .with_texture_canvas(gameplay_texture, |canvas| {
                for row in 0..30 {
                    for col in 0..32 {
                        let nametable_address = row * 32 + col + 0x2000;

                        let nametable_value = video_buffer[nametable_address];

                        let attribute_y = row / 4;
                        let attribute_x = col / 4;

                        let attribute_value = video_buffer[0x23c0 + attribute_x + attribute_y * 8];

                        let top_left = attribute_value & 0b11;
                        let top_right = attribute_value.bitand(0b1100 as u8) >> 2;
                        let bottom_left = attribute_value.bitand(0b110000 as u8) >> 4;
                        let bottom_right = attribute_value.bitand(0b11000000 as u8) >> 6;

                        let subtile_y = row % 4;
                        let subtile_x = col % 4;

                        let palette_set_index = match (subtile_x / 2, subtile_y / 2) {
                            (0, 0) => top_left,
                            (1, 0) => top_right,
                            (1, 1) => bottom_left,
                            (0, 1) => bottom_right,
                            _ => panic!("Impossible subtile location!"),
                        };

                        let xx: i32 = col.try_into().unwrap();
                        let yy: i32 = row.try_into().unwrap();

                        canvas
                            .copy(
                                &pattern_texture,
                                sdl2::rect::Rect::new(
                                    0,
                                    nametable_value as i32 * 8 + 2048 * palette_set_index as i32,
                                    8,
                                    8,
                                ),
                                sdl2::rect::Rect::new(xx * 8, yy * 8, 8, 8),
                            )
                            .unwrap();
                    }
                }

                canvas.copy(debug_texture, None, None).unwrap();
            })
            .unwrap();

        self.canvas
            .copy(
                &gameplay_texture,
                None,
                sdl2::rect::Rect::new(0, 0, 256 * SCALE, 240 * SCALE),
            )
            .unwrap();

        self.canvas.set_draw_color(Color::GREEN);

        self.canvas
            .copy(
                &self.left_pattern_texture,
                None,
                sdl2::rect::Rect::new(256 * SCALE as i32 + 10, 10, 128, 128),
            )
            .unwrap();

        self.canvas
            .fill_rect(sdl2::rect::Rect::new(
                256 * SCALE as i32 + 150,
                10,
                128,
                128,
            ))
            .unwrap();

        self.canvas.present();
    }
}
