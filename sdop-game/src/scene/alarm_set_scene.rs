use chrono::{NaiveTime, Weekday, WeekdaySet};
use fixedstr::str_format;

use crate::{
    alarm::AlarmConfig,
    display::GameDisplay,
    scene::{
        RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs,
        enter_date_scene::{self, EnterDateScene},
        home_scene::HomeScene,
        weekday_select_scene::WeekdaySelectScene,
    },
};

enum State {
    GettingTime,
    GettingDay,
    GotDay,
}

pub struct AlarmSetScene {
    state: State,
    time: Option<NaiveTime>,
    days: Option<WeekdaySet>,
}

impl AlarmSetScene {
    pub fn new() -> Self {
        Self {
            state: State::GettingTime,
            time: None,
            days: None,
        }
    }
}

impl Scene for AlarmSetScene {
    fn setup(&mut self, _args: &mut SceneTickArgs) {}

    fn teardown(&mut self, args: &mut SceneTickArgs) {
        if let Some(days) = self.days
            && let Some(time) = self.time
        {
            args.game_ctx
                .alarm
                .set_config(crate::alarm::AlarmConfig::Time {
                    days: days,
                    time: time,
                });
        }
    }

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        match self.state {
            State::GettingTime => {
                self.state = State::GettingDay;

                let time = match args.game_ctx.alarm.config() {
                    AlarmConfig::None => NaiveTime::default(),
                    AlarmConfig::Time { days: _, time } => *time,
                };

                SceneOutput::new(SceneEnum::EnterDate(
                    EnterDateScene::new(
                        enter_date_scene::Required::Time,
                        str_format!(fixedstr::str12, "ALARM TIME?"),
                    )
                    .with_time(time),
                ))
            }
            State::GettingDay => {
                self.state = State::GotDay;
                if self.time.is_none() {
                    self.time = Some(args.game_ctx.shared_out.time_out);
                }

                let days = match args.game_ctx.alarm.config() {
                    AlarmConfig::None => WeekdaySet::EMPTY,
                    AlarmConfig::Time { days, time: _ } => *days,
                };

                SceneOutput::new(SceneEnum::WeekDaySelect(
                    WeekdaySelectScene::new(1, str_format!(fixedstr::str12, "WHAT DAYS?"))
                        .with_days(days),
                ))
            }
            State::GotDay => {
                if self.days.is_none() {
                    self.days = Some(args.game_ctx.shared_out.weekday_out);
                }

                SceneOutput::new(SceneEnum::Home(HomeScene::new()))
            }
        }
    }

    fn render(&self, _display: &mut GameDisplay, _args: &mut RenderArgs) {}
}
