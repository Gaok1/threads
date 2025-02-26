use macroquad::prelude::*;
use std::sync::{Arc, RwLock};

/// Dados internos de um Recurso.
pub struct ResourceInner {
    pub name: String,
    pub pos: Vec2,
    pub width: f32,
    pub height: f32,

    /// Quantas leituras estão em uso simultaneamente
    pub read_count: u32,

    /// Quantas escritas (normalmente 0 ou 1) estão em uso
    pub write_count: u32,
}

/// Invólucro com Arc<RwLock<...>>
#[derive(Clone)]
pub struct Resource {
    pub data: Arc<RwLock<ResourceInner>>,
}

impl Resource {
    /// Cria um novo `Resource` com contadores zerados.
    pub fn new(name: &str, pos: Vec2, width: f32, height: f32) -> Self {
        let inner = ResourceInner {
            name: name.to_string(),
            pos,
            width,
            height,
            read_count: 0,
            write_count: 0,
        };
        Resource {
            data: Arc::new(RwLock::new(inner)),
        }
    }

    /// Tenta ativar leitura (retorna `true` se conseguiu).
    /// Regra: não pode haver escritor ativo.
    pub fn try_set_reading(&self) -> bool {
        if let Ok(mut inner) = self.data.write() {
            // Se já tiver writer, falha
            if inner.write_count > 0 {
                return false;
            }
            // Caso contrário, incrementa contagem de leitura
            inner.read_count += 1;
            return true;
        }
        false
    }

    /// Tenta ativar escrita (retorna `true` se conseguiu).
    /// Regra: não pode haver nenhum escritor nem leitores.
    pub fn try_set_writing(&self) -> bool {
        if let Ok(mut inner) = self.data.write() {
            // Se houver readers ou já um writer, falha
            if inner.read_count > 0 || inner.write_count > 0 {
                return false;
            }
            // Caso contrário, pode escrever
            inner.write_count += 1;
            return true;
        }
        false
    }

    /// Sai do modo de leitura (decrementa read_count).
    pub fn remove_reading(&self) {
        if let Ok(mut inner) = self.data.write() {
            if inner.read_count > 0 {
                inner.read_count -= 1;
            }
        }
    }

    /// Sai do modo de escrita (decrementa write_count).
    pub fn remove_writing(&self) {
        if let Ok(mut inner) = self.data.write() {
            if inner.write_count > 0 {
                inner.write_count -= 1;
            }
        }
    }
}

/// Uma caixa que contém vários recursos e os desenha.
pub struct ResourceBox {
    pub pos: Vec2,
    pub resources: Vec<Resource>,
}

const RESOURCE_BOX_WIDTH: f32 = 500.0;
const RESOURCE_BOX_HEIGHT: f32 = 300.0;
const RESOURCE_BOX_BORDER_SIZE: f32 = 5.0;

impl ResourceBox {
    pub fn new(pos: Vec2, resources_len: u32) -> Self {
        let usable_width = RESOURCE_BOX_WIDTH - (RESOURCE_BOX_BORDER_SIZE * 2.0);
        let usable_height = RESOURCE_BOX_HEIGHT - (RESOURCE_BOX_BORDER_SIZE * 2.0);

        let resource_width = if resources_len > 0 {
            usable_width / resources_len as f32
        } else {
            0.0
        };
        let resource_height = usable_height;

        let mut resources = Vec::with_capacity(resources_len as usize);
        for i in 0..resources_len {
            let x_offset = RESOURCE_BOX_BORDER_SIZE + (i as f32 * resource_width);
            let resource_pos = Vec2::new(pos.x + x_offset, pos.y + RESOURCE_BOX_BORDER_SIZE);

            resources.push(Resource::new(
                &format!("Resource {}", i + 1),
                resource_pos,
                resource_width,
                resource_height,
            ));
        }
        Self { pos, resources }
    }

    /// Tenta ativar leitura em `idx`. Retorna `true` se conseguiu.
    pub fn try_set_reading(&self, idx: usize) -> bool {
        if let Some(r) = self.resources.get(idx) {
            return r.try_set_reading();
        }
        false
    }

    /// Tenta ativar escrita em `idx`. Retorna `true` se conseguiu.
    pub fn try_set_writing(&self, idx: usize) -> bool {
        if let Some(r) = self.resources.get(idx) {
            return r.try_set_writing();
        }
        false
    }

    /// Sai de leitura
    pub fn remove_reading(&self, idx: usize) {
        if let Some(r) = self.resources.get(idx) {
            r.remove_reading();
        }
    }

    /// Sai de escrita
    pub fn remove_writing(&self, idx: usize) {
        if let Some(r) = self.resources.get(idx) {
            r.remove_writing();
        }
    }

    /// Desenha a caixa e seus recursos.
    pub fn draw(&self) {
        // Borda externa
        draw_rectangle_lines(
            self.pos.x,
            self.pos.y,
            RESOURCE_BOX_WIDTH,
            RESOURCE_BOX_HEIGHT,
            2.0,
            BLACK,
        );

        for resource in &self.resources {
            if let Ok(inner) = resource.data.read() {
                let readers = inner.read_count;
                let writers = inner.write_count;

                // Cor: se writers>0 => vermelho, senão se readers>0 => verde, senão cinza
                let background_color = if writers > 0 {
                    Color::new(0.9, 0.4, 0.4, 1.0)
                } else if readers > 0 {
                    Color::new(0.4, 0.8, 0.4, 1.0)
                } else {
                    Color::new(0.7, 0.7, 0.7, 1.0)
                };

                // Retângulo do recurso
                draw_rectangle(
                    inner.pos.x,
                    inner.pos.y,
                    inner.width,
                    inner.height,
                    background_color,
                );
                draw_rectangle_lines(
                    inner.pos.x,
                    inner.pos.y,
                    inner.width,
                    inner.height,
                    2.0,
                    BLACK,
                );

                // Faixa branca para texto
                let text_bg_height = 30.0;
                draw_rectangle(inner.pos.x, inner.pos.y, inner.width, text_bg_height, WHITE);

                // Nome, estado e contadores
                let resource_state_text = if writers > 0 {
                    "Writing"
                } else if readers > 0 {
                    "Reading"
                } else {
                    "Idle"
                };
                let counters_str = format!("Readers: {}, Writers: {}", readers, writers);
                let full_text = format!(
                    "{}\nState: {}\n{}",
                    inner.name, resource_state_text, counters_str
                );

                // Desenhar texto com wrap
                let font_size = 18.0;
                let left_margin = 5.0;
                let top_margin = 5.0;
                let text_start_x = inner.pos.x + left_margin;
                let text_start_y = inner.pos.y + top_margin;
                let max_text_width = inner.width - 2.0 * left_margin;
                let max_text_height = inner.height - top_margin;

                draw_wrapped_text(
                    &full_text,
                    text_start_x,
                    text_start_y,
                    max_text_width,
                    max_text_height,
                    font_size,
                    BLACK,
                );
            }
        }
    }
}

/// Desenha texto com wrap
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

            let space_width = if current_line.is_empty() {
                0.0
            } else {
                measure_text(" ", None, font_size as u16, 1.0).width
            };
            let next_width = current_line_width + space_width + word_width;

            // Se a palavra sozinha for maior que max_width, truncar
            if word_width > max_width {
                if !current_line.is_empty() {
                    draw_text_line(&current_line, start_x, cursor_y, font_size, color);
                    cursor_y += line_spacing;
                    if cursor_y > start_y + max_height {
                        return;
                    }
                }
                let mut truncated = String::new();
                for ch in word.chars() {
                    let test_str = truncated.clone() + &ch.to_string();
                    let w = measure_text(&test_str, None, font_size as u16, 1.0).width;
                    if w > max_width {
                        break;
                    }
                    truncated.push(ch);
                }
                draw_text_line(&truncated, start_x, cursor_y, font_size, color);
                cursor_y += line_spacing;
                if cursor_y > start_y + max_height {
                    return;
                }
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
                if cursor_y > start_y + max_height {
                    return;
                }
                current_line.clear();
                current_line.push_str(word);
                current_line_width = word_width;
            }
        }

        if !current_line.is_empty() {
            draw_text_line(&current_line, start_x, cursor_y, font_size, color);
            cursor_y += line_spacing;
            if cursor_y > start_y + max_height {
                return;
            }
        }
    }
}

fn draw_text_line(line: &str, x: f32, y: f32, font_size: f32, color: Color) {
    draw_text(line, x, y + font_size, font_size, color);
}
