/// A Python module implemented in Rust. The name of this module must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pyo3::pymodule]
mod sdop_py {
    use std::time::Duration;

    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    use pyo3::{PyRef, PyResult, prelude::*};
    use sdop_game::{ButtonState, SaveFile, Timestamp};

    #[pyclass]
    #[derive(Clone, Copy)]
    struct GameTime {
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    }

    #[pymethods]
    impl GameTime {
        #[new]
        pub fn new(
            year: i32,
            month: u32,
            day: u32,
            hour: u32,
            minute: u32,
            second: u32,
        ) -> PyResult<Self> {
            Ok(Self {
                year,
                month,
                day,
                hour,
                minute,
                second,
            })
        }
    }

    impl Into<Timestamp> for GameTime {
        fn into(self) -> Timestamp {
            Timestamp::new(NaiveDateTime::new(
                NaiveDate::from_ymd_opt(self.year, self.month, self.day).unwrap(),
                NaiveTime::from_hms_opt(self.hour, self.minute, self.second).unwrap(),
            ))
        }
    }

    #[pyclass]
    struct GamePy {
        game: sdop_game::Game,
    }

    #[pymethods]
    impl GamePy {
        #[new]
        fn new(time: PyRef<'_, GameTime>) -> PyResult<Self> {
            let game = sdop_game::Game::blank(Some(time.clone().into()));
            Ok(GamePy { game })
        }

        #[staticmethod]
        fn load_from_save(now: PyRef<'_, GameTime>, data: Vec<u8>) -> PyResult<Self> {
            if let Ok(save) = SaveFile::from_bytes(&data) {
                let mut game = sdop_game::Game::blank(None);
                game.load_save(now.clone().into(), save);
                return Ok(GamePy { game: game });
            }

            GamePy::new(now)
        }

        fn tick(mut self_: PyRefMut<'_, Self>, delta_nanos: u64) {
            self_.game.tick(Duration::from_nanos(delta_nanos));
        }

        fn refresh_display(mut self_: PyRefMut<'_, Self>, delta_nanos: u64) {
            self_
                .game
                .refresh_display(Duration::from_nanos(delta_nanos));
        }

        fn display_bitmap<'py>(self_: PyRef<'py, Self>) -> PyResult<Vec<u8>> {
            Ok(self_.game.get_display_bmp().iter().cloned().collect())
        }

        fn update_inputs(mut self_: PyRefMut<'_, Self>, left: bool, middle: bool, right: bool) {
            self_.game.update_input_states([
                if left {
                    ButtonState::Down
                } else {
                    ButtonState::Up
                },
                if middle {
                    ButtonState::Down
                } else {
                    ButtonState::Up
                },
                if right {
                    ButtonState::Down
                } else {
                    ButtonState::Up
                },
            ]);
        }

        fn get_save_bytes(self_: PyRef<'_, Self>, time: PyRef<'_, GameTime>) -> Option<Vec<u8>> {
            self_
                .game
                .get_save(time.clone().into())
                .and_then(|save| save.to_bytes().ok())
                .map(|bytes| bytes.into_iter().collect())
        }
    }

    /// Formats the sum of two numbers as string.
    #[pyfunction]
    fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
        Ok((a + b).to_string())
    }
}
