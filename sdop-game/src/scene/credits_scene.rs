use crate::{
    display::{CENTER_VEC, ComplexRenderOption, GameDisplay},
    fonts::FONT_VARIABLE_SMALL,
    scene::{
        RenderArgs, Scene, SceneOutput, SceneTickArgs,
    },
};

pub struct CreditsScene {}

impl Default for CreditsScene {
    fn default() -> Self {
        Self::new()
    }
}

impl CreditsScene {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scene for CreditsScene {
    fn setup(&mut self, _args: &mut SceneTickArgs) {}

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs, output: &mut SceneOutput) {
        if args.input.any_pressed() {
            output.set_home();
            return;
        }
    }

    fn render(&self, display: &mut GameDisplay, _args: &mut RenderArgs) {
        display.render_text_complex(
            CENTER_VEC,
            "CREDITS TBD",
            ComplexRenderOption::new()
                .with_white()
                .with_center()
                .with_font(&FONT_VARIABLE_SMALL),
        );

        display.invert();
    }
}
