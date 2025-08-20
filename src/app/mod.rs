use std::io::Write;

use crossbeam::channel::{Receiver, Sender};

use macroquad::{miniquad::window::quit, prelude::*};

use crate::cpu::CpuEvent;

pub struct App {
    cpu_events: Receiver<CpuEvent>,
    text_buffer: String,
    is_running: bool,
}

impl App {
    /// Creates a new [`App`] with a channel to receive CPU events.
    /// It returns a tuple containing the `App` instance and a receiver for application events.
    /// # Arguments
    /// * `cpu_events` - A receiver channel for CPU events.
    /// # Returns
    /// A tuple containing the `App` instance and a receiver for application events.
    pub fn new(cpu_events: Receiver<CpuEvent>) -> Self {
        App {
            cpu_events,
            text_buffer: String::new(),
            is_running: true,
        }
    }

    /// Runs the application.
    pub async fn run(&mut self) {
        loop {
            clear_background(macroquad::color::BLACK);
            if self.is_running {
                for event in self.cpu_events.try_iter() {
                    log::trace!("Received CPU event: {:?}", event);
                    match event {
                        CpuEvent::DrawCharacter { character } => {
                            log::debug!("Drawing character: '{}'", character.escape_debug());
                            self.text_buffer.push(character);
                        }
                        CpuEvent::Exit { exit_code } => {
                            log::debug!("Exiting with code: {}", exit_code);
                            self.text_buffer
                                .push_str(&format!("\nExiting with code: {}", exit_code));
                            self.is_running = false;
                        }
                    }
                }
            }

            draw_multiline_text(
                &self.text_buffer,
                0.0,
                16.0,
                20.0,
                None,
                macroquad::color::WHITE,
            );

            if is_key_pressed(KeyCode::Q) {
                quit();
            }

            if is_quit_requested() {
                log::info!("Quit requested by user.");
                self.is_running = false;
            }

            next_frame().await;
        }
    }
}
