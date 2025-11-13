use crate::{
    display::GameDisplay,
    scene::{
        inventory_scene::InventoryScene, RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs,
    },
};

pub struct CreditsScene {}

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

    fn render(&self, _display: &mut GameDisplay, _args: &mut RenderArgs) {}
}
