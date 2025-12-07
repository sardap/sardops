
use crate::{
    display::{ComplexRenderOption, GameDisplay, CENTER_VEC},
    fonts::FONT_VARIABLE_SMALL,
    scene::{
        inventory_scene::InventoryScene, RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs,
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

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        if args.input.any_pressed() {
            return SceneOutput::new(SceneEnum::Inventory(InventoryScene::new()));
        }

        SceneOutput::default()
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
