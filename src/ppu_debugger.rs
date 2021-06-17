use sdl2::{
    pixels::Color,
    rect::Rect,
    render::{Texture, TextureCreator, WindowCanvas},
    video::WindowContext,
};

use crate::ppu::Ppu;

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
