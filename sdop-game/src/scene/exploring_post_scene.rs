use core::time::Duration;

use fixedstr::str_format;
use glam::IVec2;

use crate::{
    assets,
    display::{
        CENTER_X_I32, ComplexRenderOption, GameDisplay, HEIGHT_I32, WIDTH_I32, WrappingMode,
    },
    fonts::FONT_VARIABLE_SMALL,
    scene::{RenderArgs, Scene, SceneOutput, SceneTickArgs},
};

pub struct ExploringPostScene {
    elapsed: Duration,
}

impl ExploringPostScene {
    pub fn new() -> Self {
        Self {
            elapsed: Duration::ZERO,
        }
    }
}

impl Scene for ExploringPostScene {
    fn setup(&mut self, _args: &mut SceneTickArgs) {}

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs, output: &mut SceneOutput) {
        self.elapsed += args.delta;

        if args.input.any_pressed() || self.elapsed > Duration::from_hours(1) {
            output.set_home();
            return;
        }
    }

    fn render(&self, display: &mut GameDisplay, args: &mut RenderArgs) {
        let result = args.game_ctx.explore_system.last_result();

        let mut y = 2;

        let completed = result.percent_passed();

        if completed > 0.05 {
            let str = str_format!(
                fixedstr::str24,
                "Passed {}% of",
                libm::roundf(completed * 100.) as i32
            );
            display.render_text_complex(
                &IVec2::new(CENTER_X_I32, y),
                &str,
                ComplexRenderOption::new()
                    .with_white()
                    .with_center()
                    .with_font(&FONT_VARIABLE_SMALL),
            );
            y += 6;
            display.render_text_complex(
                &IVec2::new(CENTER_X_I32, y),
                "hurdles",
                ComplexRenderOption::new()
                    .with_white()
                    .with_center()
                    .with_font(&FONT_VARIABLE_SMALL),
            );
            y += 8;
        } else {
            let str = str_format!(fixedstr::str24, "{} IS A", args.game_ctx.pet.name.trim());
            display.render_text_complex(
                &IVec2::new(CENTER_X_I32, y),
                &str,
                ComplexRenderOption::new()
                    .with_white()
                    .with_center()
                    .with_font(&FONT_VARIABLE_SMALL),
            );
            y += 6;
            display.render_text_complex(
                &IVec2::new(CENTER_X_I32, y),
                "COMPLETE",
                ComplexRenderOption::new()
                    .with_white()
                    .with_center()
                    .with_font(&FONT_VARIABLE_SMALL),
            );
            y += 6;
            display.render_text_complex(
                &IVec2::new(CENTER_X_I32, y),
                "FAILURE",
                ComplexRenderOption::new()
                    .with_white()
                    .with_center()
                    .with_font(&FONT_VARIABLE_SMALL),
            );
            y += 8;
        }

        if result.completed() {
            let str = str_format!(fixedstr::str24, "found ${} AND", result.earnings);
            display.render_text_complex(
                &IVec2::new(CENTER_X_I32, y),
                &str,
                ComplexRenderOption::new()
                    .with_white()
                    .with_center()
                    .with_font(&FONT_VARIABLE_SMALL),
            );

            y += 6;
            if result.items.len() == 0 {
                display.render_text_complex(
                    &IVec2::new(CENTER_X_I32, y),
                    "NOTHING ELSE",
                    ComplexRenderOption::new()
                        .with_white()
                        .with_center()
                        .with_font(&FONT_VARIABLE_SMALL),
                );
                y += 5;
            }
            for (i, item) in result.items.iter().enumerate() {
                log::info!("{} {} ", i, item.name());
                let str = str_format!(fixedstr::str32, "{}.{}", i + 1, item.name());
                let end = display.render_text_complex(
                    &IVec2::new(1, y),
                    &str,
                    ComplexRenderOption::new()
                        .with_white()
                        .with_font(&FONT_VARIABLE_SMALL)
                        .with_font_wrapping_x(WrappingMode::Partial(WIDTH_I32 - 1)),
                );
                y += (end.y - y) + 1;
            }
        } else {
            display.render_text_complex(
                &IVec2::new(CENTER_X_I32, y + 5),
                "YOU GET...",
                ComplexRenderOption::new()
                    .with_white()
                    .with_center()
                    .with_font(&FONT_VARIABLE_SMALL),
            );

            display.render_text_complex(
                &IVec2::new(CENTER_X_I32, y + 15),
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
