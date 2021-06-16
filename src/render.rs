use std::{convert::TryInto, ops::BitAnd};

use sdl2::{
    pixels::{Color, Palette, PixelFormatEnum},
    rect::Rect,
    render::{BlendMode, Texture, WindowCanvas},
    surface::Surface,
    video::Window,
};
use sdl2::{
    render::{Canvas, TextureCreator},
    video::WindowContext,
};

use crate::ppu::{DrawPriority, PatternTableSelection, Ppu};

const COLOR_KEY: Color = Color::RGBA(3, 3, 3, 3);

const PALETTE: [(u8, u8, u8, u8); 64] = [
    (0x80, 0x80, 0x80, 0xff),
    (0x00, 0x3D, 0xA6, 0xff),
    (0x00, 0x12, 0xB0, 0xff),
    (0x44, 0x00, 0x96, 0xff),
    (0xA1, 0x00, 0x5E, 0xff),
    (0xC7, 0x00, 0x28, 0xff),
    (0xBA, 0x06, 0x00, 0xff),
    (0x8C, 0x17, 0x00, 0xff),
    (0x5C, 0x2F, 0x00, 0xff),
    (0x10, 0x45, 0x00, 0xff),
    (0x05, 0x4A, 0x00, 0xff),
    (0x00, 0x47, 0x2E, 0xff),
    (0x00, 0x41, 0x66, 0xff),
    (0x00, 0x00, 0x00, 0xff),
    (0x05, 0x05, 0x05, 0xff),
    (0x05, 0x05, 0x05, 0xff),
    (0xC7, 0xC7, 0xC7, 0xff),
    (0x00, 0x77, 0xFF, 0xff),
    (0x21, 0x55, 0xFF, 0xff),
    (0x82, 0x37, 0xFA, 0xff),
    (0xEB, 0x2F, 0xB5, 0xff),
    (0xFF, 0x29, 0x50, 0xff),
    (0xFF, 0x22, 0x00, 0xff),
    (0xD6, 0x32, 0x00, 0xff),
    (0xC4, 0x62, 0x00, 0xff),
    (0x35, 0x80, 0x00, 0xff),
    (0x05, 0x8F, 0x00, 0xff),
    (0x00, 0x8A, 0x55, 0xff),
    (0x00, 0x99, 0xCC, 0xff),
    (0x21, 0x21, 0x21, 0xff),
    (0x09, 0x09, 0x09, 0xff),
    (0x09, 0x09, 0x09, 0xff),
    (0xFF, 0xFF, 0xFF, 0xff),
    (0x0F, 0xD7, 0xFF, 0xff),
    (0x69, 0xA2, 0xFF, 0xff),
    (0xD4, 0x80, 0xFF, 0xff),
    (0xFF, 0x45, 0xF3, 0xff),
    (0xFF, 0x61, 0x8B, 0xff),
    (0xFF, 0x88, 0x33, 0xff),
    (0xFF, 0x9C, 0x12, 0xff),
    (0xFA, 0xBC, 0x20, 0xff),
    (0x9F, 0xE3, 0x0E, 0xff),
    (0x2B, 0xF0, 0x35, 0xff),
    (0x0C, 0xF0, 0xA4, 0xff),
    (0x05, 0xFB, 0xFF, 0xff),
    (0x5E, 0x5E, 0x5E, 0xff),
    (0x0D, 0x0D, 0x0D, 0xff),
    (0x0D, 0x0D, 0x0D, 0xff),
    (0xFF, 0xFF, 0xFF, 0xff),
    (0xA6, 0xFC, 0xFF, 0xff),
    (0xB3, 0xEC, 0xFF, 0xff),
    (0xDA, 0xAB, 0xEB, 0xff),
    (0xFF, 0xA8, 0xF9, 0xff),
    (0xFF, 0xAB, 0xB3, 0xff),
    (0xFF, 0xD2, 0xB0, 0xff),
    (0xFF, 0xEF, 0xA6, 0xff),
    (0xFF, 0xF7, 0x9C, 0xff),
    (0xD7, 0xE8, 0x95, 0xff),
    (0xA6, 0xED, 0xAF, 0xff),
    (0xA2, 0xF2, 0xDA, 0xff),
    (0x99, 0xFF, 0xFC, 0xff),
    (0xDD, 0xDD, 0xDD, 0xff),
    (0x11, 0x11, 0x11, 0xff),
    (0x11, 0x11, 0x11, 0xff),
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

fn create_texture<'r>(
    texture_creator: &'r TextureCreator<WindowContext>,
    raw_bytes: &[u8],
    color_palette: &Palette,
) -> Texture<'r> {
    let mut pattern_surface = Surface::new(8, 256 * 8, PixelFormatEnum::Index8).unwrap();

    pattern_surface.set_color_key(true, COLOR_KEY).unwrap();

    pattern_surface.set_palette(&color_palette).unwrap();

    let pattern_surface_raw = pattern_surface.without_lock_mut().unwrap();
    pattern_surface_raw.copy_from_slice(raw_bytes);

    let pattern_texture = texture_creator
        .create_texture_from_surface(pattern_surface)
        .unwrap();

    pattern_texture
}
struct PatternBank<'r> {
    textures: Vec<Texture<'r>>,
    sprite_textures: Vec<Texture<'r>>,
}

impl<'r> PatternBank<'r> {
    fn new(
        pattern_table: &[u8],
        background_color_sets: &Vec<Palette>,
        sprite_color_sets: &Vec<Palette>,
        texture_creator: &'r TextureCreator<WindowContext>,
    ) -> PatternBank<'r> {
        /*
        Generate the pattern tiles.
        The tiles are arranged vertically and also grouped in sets.
        (0, 0) -> (7, 7): Tile 0 palette set 1
        (0, 8) -> (7, 15): Tile 0 pallette set 1
        (0, 2048) -> (7, 2055): Tile 0 pallette set 2
        */

        let mut raw_bytes = [0u8; 256 * 8 * 8];
        for index in 0..256 {
            let address = index * 0x10;

            for row in 0..8 {
                let left_bits = pattern_table[address + row];
                let right_bits = pattern_table[address + row + 8];

                for col in 0..8 {
                    let palette_value = palette_number(left_bits, right_bits, col);

                    // fill each palette set
                    raw_bytes[row * 8 + col + index * 64] = palette_value;
                }
            }
        }

        let textures: Vec<Texture> = (0..4)
            .map(|set_number| {
                create_texture(
                    &texture_creator,
                    &raw_bytes,
                    &background_color_sets[set_number],
                )
            })
            .collect();

        let sprite_textures: Vec<Texture> = (0..4)
            .map(|set_number| {
                create_texture(&texture_creator, &raw_bytes, &sprite_color_sets[set_number])
            })
            .collect();

        PatternBank {
            textures,
            sprite_textures,
        }
    }

    fn render_tile(
        &self,
        canvas: &mut WindowCanvas,
        nametable_value: u8,
        attribute_value: u8,
        dst: Rect,
    ) {
        self.render_tile_ex(canvas, nametable_value, attribute_value, dst, false, false);
    }

    fn render_tile_ex(
        &self,
        canvas: &mut WindowCanvas,
        nametable_value: u8,
        attribute_value: u8,
        dst: Rect,
        flip_horizontal: bool,
        flip_vertical: bool,
    ) {
        canvas
            .copy_ex(
                &self.textures[attribute_value as usize],
                Rect::new(0, nametable_value as i32 * 8, 8, 8),
                dst,
                0.0,
                None,
                flip_horizontal,
                flip_vertical,
            )
            .unwrap();
    }

    fn render_sprite_ex(
        &self,
        canvas: &mut WindowCanvas,
        nametable_value: u8,
        attribute_value: u8,
        dst: Rect,
        flip_horizontal: bool,
        flip_vertical: bool,
    ) {
        canvas
            .copy_ex(
                &self.sprite_textures[attribute_value as usize],
                Rect::new(0, nametable_value as i32 * 8, 8, 8),
                dst,
                0.0,
                None,
                flip_horizontal,
                flip_vertical,
            )
            .unwrap();
    }

    fn create_debug_texture(
        &self,
        canvas: &mut WindowCanvas,
        texture_creator: &'r TextureCreator<WindowContext>,
        palette_set_index: u8,
    ) -> Texture<'r> {
        let mut texture = texture_creator
            .create_texture_target(None, 128, 128)
            .unwrap();

        canvas
            .with_texture_canvas(&mut texture, |canvas| {
                canvas.set_draw_color(Color::BLACK);
                canvas.clear();

                for row in 0..16 {
                    for col in 0..16 {
                        let tile_number = row * 16 + col;

                        canvas
                            .copy(
                                &self.textures[palette_set_index as usize],
                                Rect::new(0, tile_number * 8, 8, 8),
                                Rect::new(col * 8, row * 8, 8, 8),
                            )
                            .unwrap();
                    }
                }
            })
            .unwrap();

        texture
    }
}

fn create_sdl_palette(color_palette: &[[u8; 3]]) -> Vec<Palette> {
    let palettes: Vec<Palette> = (0..4)
        .map(|set_index| {
            let color_set = color_palette;

            let mut colors = vec![COLOR_KEY];

            let set = color_set[set_index as usize];

            for color_index in 0..3 {
                let palette_index = set[color_index as usize];
                let (r, g, b, _) = PALETTE[palette_index as usize];
                colors.push(Color::RGB(r, g, b));
            }

            Palette::with_colors(&colors).unwrap()
        })
        .collect();

    palettes
}

pub struct Renderer<'a> {
    canvas: Canvas<Window>,

    texture_creator: &'a TextureCreator<WindowContext>,
    debug_texture: Texture<'a>,
    background_texture: Texture<'a>,
    gameplay_texture: Texture<'a>,
    left_pattern_texture: Texture<'a>,
    right_pattern_texture: Texture<'a>,
    foreground_sprite_texture: Texture<'a>,
    background_sprite_texture: Texture<'a>,
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

        let mut background_sprite_texture = texture_creator
            .create_texture_target(None, 256, 240)
            .unwrap();

        background_sprite_texture.set_blend_mode(BlendMode::Blend);

        let mut background_texture = texture_creator
            .create_texture_target(None, 256, 240)
            .unwrap();
        background_texture.set_blend_mode(BlendMode::Blend);

        let mut foreground_sprite_texture = texture_creator
            .create_texture_target(canvas.default_pixel_format(), 256, 240)
            .unwrap();

        foreground_sprite_texture.set_blend_mode(sdl2::render::BlendMode::Blend);

        let left_pattern_texture = texture_creator
            .create_texture_target(None, 128, 128)
            .unwrap();

        let right_pattern_texture = texture_creator
            .create_texture_target(None, 128, 128)
            .unwrap();

        Renderer {
            canvas,
            texture_creator,
            debug_texture,
            background_texture,
            gameplay_texture,
            left_pattern_texture,
            right_pattern_texture,
            foreground_sprite_texture,
            background_sprite_texture,
        }
    }

    pub fn render(&mut self, ppu: &Ppu) {
        self.canvas.set_draw_color(Color::GRAY);
        self.canvas.clear();

        let video_buffer = ppu.get_buffer();

        let raw_palette = ppu.get_color_palette();

        let background_color_sets = create_sdl_palette(&raw_palette.background_color_set);
        let sprite_color_sets = create_sdl_palette(&raw_palette.sprite_color_set);

        let start_time = std::time::SystemTime::now();

        let left_pattern_bank = PatternBank::new(
            ppu.left_pattern_table(),
            &background_color_sets,
            &sprite_color_sets,
            self.texture_creator,
        );

        let right_pattern_bank = PatternBank::new(
            ppu.right_pattern_table(),
            &background_color_sets,
            &sprite_color_sets,
            self.texture_creator,
        );

        // // println!("Tile generation duration: {:?}", duration);

        // // let pattern_texture =
        // //     left_pattern_bank.create_debug_texture(&mut self.canvas, self.texture_creator, 0);

        // // self.canvas
        // //     .with_texture_canvas(&mut self.left_pattern_texture, |canvas| {
        // //         canvas.copy(&pattern_texture, None, None).unwrap();
        // //     })
        // //     .unwrap();

        // // let pattern_texture =
        // //     right_pattern_bank.create_debug_texture(&mut self.canvas, self.texture_creator, 0);

        // // self.canvas
        // //     .with_texture_canvas(&mut self.right_pattern_texture, |canvas| {
        // //         canvas.copy(&pattern_texture, None, None).unwrap();
        // //     })
        // //     .unwrap();
        let background_sprite_texture = &mut self.background_sprite_texture;

        self.canvas
            .with_texture_canvas(background_sprite_texture, |canvas| {
                canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
                canvas.clear();

                for sprite_data in ppu.get_oam_sprite_data() {
                    if sprite_data.draw_priority == DrawPriority::Foreground {
                        continue;
                    }

                    let pattern_bank = match sprite_data.tile_pattern {
                        PatternTableSelection::Left => &left_pattern_bank,
                        PatternTableSelection::Right => &right_pattern_bank,
                    };

                    pattern_bank.render_sprite_ex(
                        canvas,
                        sprite_data.tile_number,
                        sprite_data.color_palette,
                        Rect::new(sprite_data.x as i32, sprite_data.y as i32, 8, 8),
                        sprite_data.flip_horizontal,
                        sprite_data.flip_vertical,
                    )
                }
            })
            .unwrap();

        let foreground_sprite_texture = &mut self.foreground_sprite_texture;
        self.canvas
            .with_texture_canvas(foreground_sprite_texture, |canvas| {
                canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
                canvas.clear();

                for sprite_data in ppu.get_oam_sprite_data() {
                    if sprite_data.draw_priority == DrawPriority::Background {
                        continue;
                    }

                    let pattern_bank = match sprite_data.tile_pattern {
                        PatternTableSelection::Left => &left_pattern_bank,
                        PatternTableSelection::Right => &right_pattern_bank,
                    };

                    pattern_bank.render_sprite_ex(
                        canvas,
                        sprite_data.tile_number,
                        sprite_data.color_palette,
                        Rect::new(sprite_data.x as i32, sprite_data.y as i32, 8, 8),
                        sprite_data.flip_horizontal,
                        sprite_data.flip_vertical,
                    )
                }
            })
            .unwrap();

        let debug_texture = &self.debug_texture;
        let background_texture = &mut self.background_texture;

        let mut background_tile_pixels = [0u8; 256 * 240 * 4];
        let frame = ppu.get_frame_buffer();

        let gameplay_texture = &mut self.gameplay_texture;

        for y in 0..240 {
            for x in 0..256 {
                // dbg!(&index, &frame[index]);
                let (r, g, b, a) = if frame[y][x] != 0xff {
                    PALETTE[frame[y][x] as usize]
                } else {
                    (0, 0, 0, 0)
                };

                let start_index = (y * 256 + x) * 4;

                background_tile_pixels[start_index] = b;
                background_tile_pixels[start_index + 1] = g;
                background_tile_pixels[start_index + 2] = r;
                background_tile_pixels[start_index + 3] = a;
            }
        }

        background_texture
            .update(Rect::new(0, 0, 256, 240), &background_tile_pixels, 256 * 4)
            .unwrap();

        // let current_pattern_bank = match ppu.current_background_pattern_table() {
        //     PatternTableSelection::Left => &left_pattern_bank,
        //     PatternTableSelection::Right => &right_pattern_bank,
        // };

        self.canvas
            .with_texture_canvas(gameplay_texture, |canvas| {
                let (r, g, b, a) = PALETTE[raw_palette.background as usize];

                canvas.set_draw_color(Color::RGBA(r, g, b, a));
                canvas.clear();

                canvas.copy(&background_sprite_texture, None, None).unwrap();
                canvas.copy(background_texture, None, None).unwrap();
                canvas.copy(debug_texture, None, None).unwrap();

                canvas.copy(foreground_sprite_texture, None, None).unwrap();
            })
            .unwrap();

        self.canvas
            .copy(
                &gameplay_texture,
                None,
                sdl2::rect::Rect::new(0, 0, 256 * SCALE, 240 * SCALE),
            )
            .unwrap();

        self.canvas
            .copy(
                &self.left_pattern_texture,
                None,
                sdl2::rect::Rect::new(256 * SCALE as i32 + 10, 10, 128, 128),
            )
            .unwrap();

        self.canvas
            .copy(
                &self.right_pattern_texture,
                None,
                Rect::new(256 * SCALE as i32 + 150, 10, 128, 128),
            )
            .unwrap();

        self.canvas.present();
    }
}
