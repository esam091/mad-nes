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

        let video_buffer = ppu.get_buffer();

        let raw_palette = ppu.get_color_palette();
        let background_color_sets = create_sdl_palette(&raw_palette.background_color_set);
        let sprite_color_sets = create_sdl_palette(&raw_palette.sprite_color_set);

        let tables = ppu.pattern_tables();
        let (left_pattern_table, right_pattern_table) = tables.get_tables();
        let left_pattern_bank = PatternBank::new(
            left_pattern_table,
            &background_color_sets,
            &sprite_color_sets,
            self.texture_creator,
        );

        let right_pattern_bank = PatternBank::new(
            right_pattern_table,
            &background_color_sets,
            &sprite_color_sets,
            self.texture_creator,
        );

        let current_pattern_bank = match ppu.current_background_pattern_table() {
            PatternTableSelection::Left => &left_pattern_bank,
            PatternTableSelection::Right => &right_pattern_bank,
        };

        render_debug_nametable(
            canvas,
            &mut self.top_left_nametable,
            ppu,
            ppu.top_left_nametable_address(),
            &current_pattern_bank,
        );

        render_debug_nametable(
            canvas,
            &mut self.top_right_nametable,
            ppu,
            ppu.top_right_nametable_address(),
            &current_pattern_bank,
        );

        render_debug_nametable(
            canvas,
            &mut self.bottom_left_nametable,
            ppu,
            ppu.bottom_left_nametable_address(),
            &current_pattern_bank,
        );

        render_debug_nametable(
            canvas,
            &mut self.bottom_right_nametable,
            ppu,
            ppu.bottom_right_nametable_address(),
            &current_pattern_bank,
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

fn render_debug_nametable(
    canvas: &mut WindowCanvas,
    texture: &mut Texture,
    ppu: &Ppu,
    nametable_address: u16,
    current_pattern_bank: &PatternBank,
) {
    let raw_palette = ppu.get_color_palette();
    let video_buffer = ppu.get_buffer();

    canvas
        .with_texture_canvas(texture, |canvas| {
            let (r, g, b, _) = PALETTE[raw_palette.background as usize];

            canvas.set_draw_color(Color::RGB(r, g, b));
            canvas.clear();

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

                    let xx: i32 = col.try_into().unwrap();
                    let yy: i32 = row.try_into().unwrap();

                    current_pattern_bank.render_tile(
                        canvas,
                        nametable_value,
                        palette_set_index,
                        Rect::new(xx * 8, yy * 8, 8, 8),
                    );
                }
            }

            // canvas.copy(debug_texture, None, None).unwrap();
        })
        .unwrap();
}
