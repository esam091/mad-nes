use std::{convert::TryInto, ops::BitAnd};

use sdl2::{
    pixels::Color,
    rect::Rect,
    render::{Texture, TextureCreator, WindowCanvas},
    video::WindowContext,
};

use crate::{
    ppu::{PatternTableSelection, Ppu},
    render::{create_sdl_palette, PatternBank, PALETTE},
};

fn create_screen_texture(texture_creator: &TextureCreator<WindowContext>) -> Texture<'_> {
    texture_creator
        .create_texture_target(None, 256, 240)
        .unwrap()
}

pub struct PpuDebugger<'a> {
    canvas: WindowCanvas,
    texture_creator: &'a TextureCreator<WindowContext>,

    top_left_nametable: Texture<'a>,
    top_right_nametable: Texture<'a>,
    bottom_left_nametable: Texture<'a>,
    bottom_right_nametable: Texture<'a>,
}

impl<'a> PpuDebugger<'a> {
    pub fn render(&mut self, ppu: &Ppu) {
        let canvas = &mut self.canvas;

        canvas.set_draw_color(Color::WHITE);
        canvas.clear();

        render_debug_nametable(
            &mut self.top_left_nametable,
            ppu,
            ppu.top_left_nametable_address(),
        );

        render_debug_nametable(
            &mut self.top_right_nametable,
            ppu,
            ppu.top_right_nametable_address(),
        );

        render_debug_nametable(
            &mut self.bottom_left_nametable,
            ppu,
            ppu.bottom_left_nametable_address(),
        );

        render_debug_nametable(
            &mut self.bottom_right_nametable,
            ppu,
            ppu.bottom_right_nametable_address(),
        );

        canvas
            .copy(&self.top_left_nametable, None, game_size_rect(0, 0))
            .unwrap();
        canvas
            .copy(&self.top_right_nametable, None, game_size_rect(389, 0))
            .unwrap();

        canvas
            .copy(&self.bottom_left_nametable, None, game_size_rect(0, 365))
            .unwrap();

        canvas
            .copy(&self.bottom_right_nametable, None, game_size_rect(389, 365))
            .unwrap();

        canvas.present();
    }

    pub fn new(
        canvas: WindowCanvas,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> PpuDebugger {
        PpuDebugger {
            canvas,
            texture_creator,
            top_left_nametable: create_screen_texture(&texture_creator),
            top_right_nametable: create_screen_texture(&texture_creator),
            bottom_left_nametable: create_screen_texture(&texture_creator),
            bottom_right_nametable: create_screen_texture(&texture_creator),
        }
    }
}

fn game_size_rect(x: i32, y: i32) -> Rect {
    Rect::new(x, y, 384, 360)
}

fn render_debug_nametable(texture: &mut Texture, ppu: &Ppu, nametable_address: u16) {
    let mut color_buffer = [[0u8; 256]; 240];

    let raw_palette = ppu.get_color_palette();
    let video_buffer = ppu.get_buffer();

    let current_nametable = nametable_address as usize;
    let current_attribute_table = current_nametable + 0x3c0;

    for row in 0..30 {
        for col in 0..32 {
            let nametable_address = row * 32 + col + current_nametable;

            let nametable_value = video_buffer[nametable_address];

            let attribute_y = row / 4;
            let attribute_x = col / 4;

            let attribute_value =
                video_buffer[current_attribute_table + attribute_x + attribute_y * 8];

            let top_left = attribute_value & 0b11;
            let top_right = attribute_value.bitand(0b1100 as u8) >> 2;
            let bottom_left = attribute_value.bitand(0b110000 as u8) >> 4;
            let bottom_right = attribute_value.bitand(0b11000000 as u8) >> 6;

            let subtile_y = row % 4;
            let subtile_x = col % 4;

            let palette_set_index = match (subtile_x / 2, subtile_y / 2) {
                (0, 0) => top_left,
                (1, 0) => top_right,
                (0, 1) => bottom_left,
                (1, 1) => bottom_right,
                _ => panic!("Impossible subtile location!"),
            };

            for y in 0..8 {
                for x in 0..8 {
                    let pattern_value = ppu.read_pattern_value(
                        ppu.current_background_pattern_table(),
                        nametable_value,
                        x,
                        y,
                    );

                    color_buffer[row * 8 + y as usize][col * 8 + x as usize] = if pattern_value == 0
                    {
                        raw_palette.background
                    } else {
                        raw_palette.background_color_set[palette_set_index as usize]
                            [pattern_value as usize - 1]
                    };
                }
            }
        }
    }

    let mut texture_buffer = [0u8; 256 * 240 * 4];

    for y in 0..240 {
        for x in 0..256 {
            let (r, g, b, a) = PALETTE[color_buffer[y][x] as usize];

            let start_index = (y * 256 + x) * 4;

            texture_buffer[start_index] = b;
            texture_buffer[start_index + 1] = g;
            texture_buffer[start_index + 2] = r;
            texture_buffer[start_index + 3] = a;
        }
    }

    texture
        .update(Rect::new(0, 0, 256, 240), &texture_buffer, 256 * 4)
        .unwrap();
}
