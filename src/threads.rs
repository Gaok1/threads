use macroquad::prelude::*;
use ::rand::random_range;

use crate::resource_box::ResourceBox;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThreadState {
    Reading,
    Writing,
    Waiting,
    Idle,
}

#[derive(Clone, Debug)]
pub struct ThreadInfo {
    pub name: String,
    pub state: ThreadState,
    pub resource_in_use: Option<usize>,
}

#[derive(Debug)]
pub struct ThreadsVisualizer {
    pub threads: Vec<ThreadInfo>,
    pub vertical_end_ratio: f32,
    pub horizontal_half_length: f32,
    pub thread_line_length: f32,
}

impl ThreadsVisualizer {
    pub fn new(num_threads: usize) -> Self {
        let mut threads = Vec::with_capacity(num_threads);
        for i in 0..num_threads {
            threads.push(ThreadInfo {
                name: format!("Thread {}", i + 1),
                state: ThreadState::Idle,
                resource_in_use: None,
            });
        }
        Self {
            threads,
            vertical_end_ratio: 0.45,
            horizontal_half_length: 800.0,
            thread_line_length: 100.0,
        }
    }

    /// Desenha as threads (com bounding box e wrap)
    pub fn draw(&self) {
        let sw = screen_width();
        let sh = screen_height();

        let center_x = sw * 0.5;
        let vertical_end_y = sh * self.vertical_end_ratio;

        // Linha vertical
        draw_line(center_x, 0.0, center_x, vertical_end_y, 3.0, BLACK);

        // Linha horizontal
        let left_x = center_x - self.horizontal_half_length;
        let right_x = center_x + self.horizontal_half_length;
        draw_line(left_x, vertical_end_y, right_x, vertical_end_y, 3.0, BLACK);

        let n = self.threads.len();
        if n == 0 {
            return;
        }

        let segment_width = (self.horizontal_half_length * 2.0) / (n as f32 + 1.0);
        let text_box_width = 180.0;
        let text_box_height = 200.0;

        for (i, thread_info) in self.threads.iter().enumerate() {
            let x_fio = left_x + segment_width * (i as f32 + 1.0);
            let y_top = vertical_end_y;
            let y_bottom = vertical_end_y + self.thread_line_length;

            // Fio
            draw_line(x_fio, y_top, x_fio, y_bottom, 2.0, BLACK);

            // Cor / texto
            let (state_text, state_color) = match thread_info.state {
                ThreadState::Reading => ("Reading", GREEN),
                ThreadState::Writing => ("Writing", RED),
                ThreadState::Waiting => ("Waiting", ORANGE),
                ThreadState::Idle => ("Idle", GRAY),
            };

            // Círculo
            let mid_y = (y_top + y_bottom) * 0.5;
            draw_circle(x_fio, mid_y, 8.0, state_color);

            // Recurso
            let resource_str = if let Some(res_idx) = thread_info.resource_in_use {
                format!("(R{})", res_idx + 1)
            } else {
                "".to_string()
            };

            let combined_text = format!("{} {}\n{}", thread_info.name, resource_str, state_text);

            let box_x = x_fio - (text_box_width * 0.5);
            let box_y = y_bottom + 20.0;

            draw_wrapped_text(
                &combined_text,
                box_x,
                box_y,
                text_box_width,
                text_box_height,
                18.0,
                BLACK,
            );
        }
    }

    /// Se a thread estava lendo/escrevendo, removemos do recurso antigo.
    /// Tentamos setar o novo estado (Reading/Writing). Se falhar (retorno false), a thread fica WAITING.
    pub fn set_thread_resource_state(
        &mut self,
        resource_box: &ResourceBox,
        index: usize,
        new_state: ThreadState,
        new_resource: Option<usize>,
    ) {
        if let Some(thread) = self.threads.get_mut(index) {
            if let Some(old_res) = thread.resource_in_use {
                match thread.state {
                    ThreadState::Reading => resource_box.remove_reading(old_res),
                    ThreadState::Writing => resource_box.remove_writing(old_res),
                    _ => {}
                }
            }
            thread.resource_in_use = None;
            thread.state = new_state;

            if let Some(res_idx) = new_resource {
                match new_state {
                    ThreadState::Reading => {
                        let ok = resource_box.try_set_reading(res_idx);
                        if ok {
                            thread.resource_in_use = Some(res_idx);
                        } else {
                            // Falhou => fica WAITING
                            thread.state = ThreadState::Waiting;
                        }
                    }
                    ThreadState::Writing => {
                        let ok = resource_box.try_set_writing(res_idx);
                        if ok {
                            thread.resource_in_use = Some(res_idx);
                        } else {
                            thread.state = ThreadState::Waiting;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// Sorteio aleatório do estado + recurso
    pub fn update_threads_randomly(&mut self, resource_box: &ResourceBox) {
       
        let total = resource_box.resources.len();
        if total == 0 {
            return;
        }
        for i in 0..self.threads.len() {
            let roll = random_range(0..4);
            let new_state = match roll {
                0 => ThreadState::Idle,
                1 => ThreadState::Waiting,
                2 => ThreadState::Reading,
                3 => ThreadState::Writing,
                _ => ThreadState::Idle,
            };
            let new_res = if new_state == ThreadState::Reading || new_state == ThreadState::Writing {
                Some(random_range(0..total))
            } else {
                None
            };
            self.set_thread_resource_state(resource_box, i, new_state, new_res);
        }
    }
}

// Mesma lógica de wrap
fn draw_wrapped_text(
    text: &str,
    start_x: f32,
    start_y: f32,
    max_width: f32,
    max_height: f32,
    font_size: f32,
    color: Color,
) {
    let forced_lines: Vec<&str> = text.split('\n').collect();
    let line_spacing = font_size + 5.0;
    let mut cursor_y = start_y;

    for line in forced_lines {
        let words: Vec<&str> = line.split_whitespace().collect();
        let mut current_line = String::new();
        let mut current_line_width = 0.0;

        for word in words {
            let measure = measure_text(word, None, font_size as u16, 1.0);
            let word_width = measure.width;

            let space_width = if current_line.is_empty() { 0.0 } else {
                measure_text(" ", None, font_size as u16, 1.0).width
            };
            let next_width = current_line_width + space_width + word_width;

            if word_width > max_width {
                if !current_line.is_empty() {
                    draw_text_line(&current_line, start_x, cursor_y, font_size, color);
                    cursor_y += line_spacing;
                    if cursor_y > start_y + max_height { return; }
                }
                let mut truncated = String::new();
                for ch in word.chars() {
                    let test = truncated.clone() + &ch.to_string();
                    let w = measure_text(&test, None, font_size as u16, 1.0).width;
                    if w > max_width {
                        break;
                    }
                    truncated.push(ch);
                }
                draw_text_line(&truncated, start_x, cursor_y, font_size, color);
                cursor_y += line_spacing;
                if cursor_y > start_y + max_height { return; }
                current_line.clear();
                current_line_width = 0.0;
                continue;
            }

            if next_width <= max_width {
                if !current_line.is_empty() {
                    current_line.push(' ');
                }
                current_line.push_str(word);
                current_line_width = next_width;
            } else {
                draw_text_line(&current_line, start_x, cursor_y, font_size, color);
                cursor_y += line_spacing;
                if cursor_y > start_y + max_height { return; }
                current_line.clear();
                current_line.push_str(word);
                current_line_width = word_width;
            }
        }

        if !current_line.is_empty() {
            draw_text_line(&current_line, start_x, cursor_y, font_size, color);
            cursor_y += line_spacing;
            if cursor_y > start_y + max_height { return; }
        }
    }
}

fn draw_text_line(line: &str, x: f32, y: f32, font_size: f32, color: Color) {
    draw_text(line, x, y + font_size, font_size, color);
}
