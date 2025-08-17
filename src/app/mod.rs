use crossbeam::channel::Receiver;

use macroquad::prelude::*;

use crate::cpu::CpuEvent;

pub struct App {
    cpu_events: Receiver<CpuEvent>,
    text_buffer: String,
}

impl App {
    /// Creates a new [`App`].
    pub fn new(cpu_events: Receiver<CpuEvent>) -> Self {
        App {
            cpu_events,
            text_buffer: String::new(),
        }
    }

    /// Runs the application.
    pub async fn run(&mut self) {
        loop {
            macroquad::window::clear_background(macroquad::color::BLACK);
            if let Some(event) = self.cpu_events.try_recv().ok() {
                log::trace!("Received CPU event: {:?}", event);
                match event {
                    CpuEvent::DrawCharacter { character } => {
                        log::info!("Drawing character: '{}'", character.escape_debug());
                        self.text_buffer.push(character);
                    }
                    CpuEvent::Exit { exit_code } => {
                        self.text_buffer
                            .push_str(&format!("\nExiting with code: {}", exit_code));
                        log::info!("Exiting with code: {}", exit_code);
                        break;
                    }
                }
            }
            macroquad::text::draw_multiline_text(
                &self.text_buffer,
                0.0,
                16.0,
                20.0,
                None,
                macroquad::color::WHITE,
            );
            next_frame().await;
        }
    }
}
