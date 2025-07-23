use crate::scene::SceneTickArgs;

pub fn tick_sim(time_scale: f32, args: &mut SceneTickArgs) {
    let delta = args.delta.mul_f32(time_scale);
    let pet = &mut args.game_ctx.pet;

    pet.tick_age(delta);
    pet.tick_hunger(delta);
}
