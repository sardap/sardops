use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::Color,
    symbols::Marker,
    widgets::{
        Widget,
        canvas::{Canvas, Rectangle},
    },
};
use sdop_game::{Game, Timestamp};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

fn timestamp() -> Timestamp {
    Timestamp::new(chrono::Local::now().naive_local())
}

/// The main application which holds the state and logic of the application.
pub struct App {
    /// Is the application running?
    running: bool,
    game: Game,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        let game = Game::new(timestamp());

        Self {
            running: true,
            game,
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            self.game.tick(timestamp());
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    ///
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/main/ratatui-widgets/examples>
    fn render(&mut self, frame: &mut Frame) {
        self.game.refresh_display(timestamp());
        // let title = Line::from("Ratatui Simple Template")
        //     .bold()
        //     .blue()
        //     .centered();
        // let text = "Hello, Ratatui!\n\n\
        //     Created using https://github.com/ratatui/templates\n\
        //     Press `Esc`, `Ctrl-C` or `q` to stop running.";
        // frame.render_widget(
        //     Paragraph::new(text)
        //         .block(Block::bordered().title(title))
        //         .centered(),
        //     frame.area(),
        // );
        let horizontal =
            Layout::horizontal([Constraint::Percentage(100), Constraint::Percentage(100)]);
        frame.render_widget(self.game_canvas(frame.area()), frame.area());
    }

    fn game_canvas(&self, area: Rect) -> impl Widget {
        let left = 0.0;
        let right = f64::from(area.width);
        let bottom = 0.0;
        let top = f64::from(area.height).mul_add(2.0, -4.0);
        Canvas::default()
            .marker(Marker::HalfBlock)
            .x_bounds([left, right])
            .y_bounds([bottom, top])
            .paint(|ctx| {
                const SCALE: usize = 1;
                for (byte_index, byte_value) in
                    self.game.get_display_image_data().iter().enumerate()
                {
                    let start_x = (byte_index % (sdop_game::WIDTH as usize / 8)) * 8;
                    let y = byte_index / (sdop_game::WIDTH as usize / 8);
                    for bit_index in 0..8 {
                        let x = start_x + bit_index;

                        let rotated_x = x;
                        let rotated_y = y;

                        let screen_x = (rotated_x * SCALE) as i32;
                        let screen_y = (rotated_y * SCALE) as i32;

                        if screen_x >= 0
                            && screen_x + 2 < sdop_game::WIDTH as i32
                            && screen_y >= 0
                            && screen_y + 2 < sdop_game::HEIGHT as i32
                        {
                            let screen_x = screen_x;
                            let screen_y = screen_y;

                            let color = if (byte_value >> (7 - bit_index)) & 1 == 1 {
                                Color::White
                            } else {
                                Color::Black
                            };

                            ctx.draw(&Rectangle {
                                x: screen_x as f64,
                                y: screen_y as f64,
                                width: f64::from(1),
                                height: f64::from(1),
                                color: color,
                            });
                        }
                    }
                }
            })
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        if let Ok(event) = event::poll(Duration::from_millis(5)) {
            if !event {
                return Ok(());
            }
            match event::read()? {
                // it's important to check KeyEventKind::Press to avoid handling key release events
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}
