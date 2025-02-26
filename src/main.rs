use std::default::Default;
use macroquad::{
    prelude::*,
    window::{Conf, next_frame, clear_background},
};

// Importamos nossos módulos
mod resource_box;
mod threads;

use resource_box::ResourceBox;
use threads::{ThreadsVisualizer, ThreadState};

/// Configuração da janela
pub fn screen_config() -> Conf {
    Conf {
        window_resizable: false,
        fullscreen: true,
        window_title: "threads".to_string(),
        ..Default::default()
    }
}

#[macroquad::main(screen_config)]
async fn main() {
    let resource_box = ResourceBox::new(vec2(50.0, 50.0), 5);

    let mut threads_vis = ThreadsVisualizer::new(8);

    let mut last_update_time = 0.0;
    let update_interval = 2.0; // a cada 2 segundos, as threads podem mudar

    loop {
        clear_background(WHITE);

        // Desenhar a ResourceBox
        resource_box.draw();

        // Desenhar as threads
        threads_vis.draw();

        // Chamar a função de atualização aleatória depois de "update_interval" segundos
        let now = get_time();
        if now - last_update_time >= update_interval {
            threads_vis.update_threads_randomly(&resource_box);
            last_update_time = now;
        }

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        next_frame().await;
    }
}
