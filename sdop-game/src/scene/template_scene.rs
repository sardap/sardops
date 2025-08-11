use crate::{
    display::GameDisplay,
    game_context::GameContext,
    scene::{Scene, SceneOutput, SceneTickArgs},
};

pub struct TemplateScene {}

impl TemplateScene {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scene for TemplateScene {
    fn setup(&mut self, _args: &mut SceneTickArgs) {}

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, _args: &mut SceneTickArgs) -> SceneOutput {
        SceneOutput::default()
    }

    fn render(&self, _display: &mut GameDisplay, _args: &mut RenderArgs) {}
}
