use fixedstr::str_format;
use glam::Vec2;

use crate::{
    assets,
    display::{CENTER_X, ComplexRenderOption, GameDisplay, HEIGHT_I32},
    fonts::FONT_VARIABLE_SMALL,
    scene::{RenderArgs, Scene, SceneOutput, SceneTickArgs},
};

pub struct ExploringPostScene {}

impl ExploringPostScene {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scene for ExploringPostScene {
    fn setup(&mut self, _args: &mut SceneTickArgs) {}

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs, output: &mut SceneOutput) {
        if args.input.any_pressed() {
            output.set_home();
            return;
        }
    }

    fn render(&self, display: &mut GameDisplay, args: &mut RenderArgs) {
        let result = args.game_ctx.explore_system.last_result();

        let mut y = 8;
        display.render_text_complex(
            Vec2::new(CENTER_X, y as f32),
            "BACK HOME",
            ComplexRenderOption::new()
                .with_white()
                .with_center()
                .with_font(&FONT_VARIABLE_SMALL),
        );
        y += 8;

        let completed = result.percent_passed();
        let str = str_format!(
            fixedstr::str24,
            "Passed {}% of",
            libm::roundf(completed * 100.) as i32
        );
        display.render_text_complex(
            Vec2::new(CENTER_X, y as f32),
            &str,
            ComplexRenderOption::new()
                .with_white()
                .with_center()
                .with_font(&FONT_VARIABLE_SMALL),
        );
        y += 6;
        display.render_text_complex(
            Vec2::new(CENTER_X, y as f32),
            "hurdles",
            ComplexRenderOption::new()
                .with_white()
                .with_center()
                .with_font(&FONT_VARIABLE_SMALL),
        );

        y += 8;
        if result.completed() {
            let str = str_format!(
                fixedstr::str24,
                "found ${} AND",
                libm::roundf(completed * 100.) as i32
            );
            display.render_text_complex(
                Vec2::new(CENTER_X, y as f32),
                &str,
                ComplexRenderOption::new()
                    .with_white()
                    .with_center()
                    .with_font(&FONT_VARIABLE_SMALL),
            );

            y += 6;
            if result.items.len() == 0 {
                display.render_text_complex(
                    Vec2::new(CENTER_X, y as f32),
                    "NOTHING ELSE",
                    ComplexRenderOption::new()
                        .with_white()
                        .with_center()
                        .with_font(&FONT_VARIABLE_SMALL),
                );
                y += 5;
            }
            for (i, item) in result.items.iter().enumerate() {
                let str = str_format!(fixedstr::str24, "{}.{}", i + 1, item.name());
                display.render_text_complex(
                    Vec2::new(1., y as f32 + (i as f32 * 6.)),
                    &str,
                    ComplexRenderOption::new()
                        .with_white()
                        .with_font(&FONT_VARIABLE_SMALL),
                );
            }
        } else {
            display.render_text_complex(
                Vec2::new(CENTER_X, y as f32 + 5.),
                "YOU GET...",
                ComplexRenderOption::new()
                    .with_white()
                    .with_center()
                    .with_font(&FONT_VARIABLE_SMALL),
            );

            display.render_text_complex(
                Vec2::new(CENTER_X, y as f32 + 15.),
                "NOTHING",
                ComplexRenderOption::new()
                    .with_white()
                    .with_center()
                    .with_font(&FONT_VARIABLE_SMALL),
            );
        }

        display.render_image_complex(
            0,
            HEIGHT_I32,
            &result.location.cover,
            ComplexRenderOption::new().with_white().with_bottom_left(),
        );
        display.render_image_complex(
            0,
            HEIGHT_I32,
            &assets::IMAGE_LOCATION_COMPLETED_X,
            ComplexRenderOption::new().with_white().with_bottom_left(),
        );
    }
}
