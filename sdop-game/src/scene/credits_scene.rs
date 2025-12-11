use crate::{
    display::{CENTER_VEC, ComplexRenderOption, GameDisplay},
    fonts::FONT_VARIABLE_SMALL,
    scene::{
        RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs, inventory_scene::InventoryScene,
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
