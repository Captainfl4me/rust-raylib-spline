use raylib::core::texture::Image;
use raylib::prelude::*;
use std::ffi::CStr;

mod bezier;
mod scenes;

mod colors;
use colors::*;

fn main() {
    let (mut rl_handle, rl_thread) = raylib::init()
        .size(640, 480)
        .resizable()
        .title("Spline drawer")
        .build();
    rl_handle.set_target_fps(60);
    rl_handle.set_exit_key(None);
    rl_handle.gui_set_style(
        GuiControl::DEFAULT,
        GuiDefaultProperty::TEXT_SIZE as i32,
        18,
    );
    rl_handle.set_window_state(rl_handle.get_window_state().set_window_maximized(true));

    let image_bytes = include_bytes!("../assets/background_tile.png");
    let mut background_tile_image = Image::load_image_from_mem(".png", image_bytes).unwrap();
    background_tile_image.resize(256, 256);
    let background_tile_texture = rl_handle
        .load_texture_from_image(&rl_thread, &background_tile_image)
        .unwrap();

    let mut scenes: Vec<Box<dyn scenes::Scene>> = vec![
        Box::<scenes::BezierCurveScene>::default(),
        Box::<scenes::BezierSplineScene>::default(),
    ];
    let mut current_scene: Option<usize> = None;

    const TITLE_FONT_SIZE: i32 = 80;
    let mut clock_divider = 0;
    let mut current_draw_time_text = String::new();
    // let mut scene_to_load = None;
    while !rl_handle.window_should_close() {
        if let Some(scene_id) = current_scene {
            scenes[scene_id].update(&mut rl_handle);
        }

        if rl_handle.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            current_scene = None;
        }
        let help_page_requested = rl_handle.is_key_down(KeyboardKey::KEY_H);

        // Draw frame
        {
            let mut rl_draw_handle = rl_handle.begin_drawing(&rl_thread);
            let screen_width = rl_draw_handle.get_screen_width();
            let screen_height = rl_draw_handle.get_screen_height();
            rl_draw_handle.clear_background(COLOR_DARK);
            draw_background(&mut rl_draw_handle, &background_tile_texture);

            if let Some(scene_id) = current_scene {
                let draw_time_start = instant::Instant::now();
                scenes[scene_id].draw(&mut rl_draw_handle);

                // Draw draw time at smaller pace than actual FPS to be able to read it.
                if clock_divider == 0 {
                    current_draw_time_text = format!("{}ms", draw_time_start.elapsed().as_millis());
                }
                rl_draw_handle.draw_text(
                    current_draw_time_text.as_str(),
                    screen_width
                        - rl_draw_handle.measure_text(current_draw_time_text.as_str(), 14)
                        - 30,
                    20,
                    18,
                    COLOR_LIGHT,
                );

                if help_page_requested {
                    draw_help_page(&mut rl_draw_handle, scenes[scene_id].help_text());
                }
            } else {
                rl_draw_handle.draw_text(
                    "Spline Drawer",
                    (screen_width - rl_draw_handle.measure_text("Spline Drawer", TITLE_FONT_SIZE))
                        / 2,
                    (screen_height - TITLE_FONT_SIZE) / 2,
                    TITLE_FONT_SIZE,
                    COLOR_LIGHT,
                );

                let load_scenes_text_list = scenes
                    .iter()
                    .map(|s| format!("Load: {}", s.get_title()))
                    .collect::<Vec<_>>();
                let max_button_width = load_scenes_text_list
                    .iter()
                    .map(|text| rl_draw_handle.measure_text(text, 30))
                    .max()
                    .unwrap();

                for (i, mut scene_name) in load_scenes_text_list.into_iter().enumerate() {
                    scene_name.push('\0');
                    if rl_draw_handle.gui_button(
                        Rectangle::new(
                            (screen_width - max_button_width) as f32 / 2.0,
                            (screen_height + TITLE_FONT_SIZE) as f32 / 2.0
                                + 10.0
                                + (40 * (i as i32)) as f32,
                            max_button_width as f32 + 20.0,
                            30.0,
                        ),
                        Some(CStr::from_bytes_with_nul(scene_name.as_bytes()).unwrap()),
                    ) {
                        current_scene = Some(i);
                    }
                }
            }
        }
        clock_divider += 1;
        if clock_divider >= 60 {
            clock_divider = 0;
        }
    }
}

fn draw_background(rl_draw_handle: &mut RaylibDrawHandle, tile_texture: &Texture2D) {
    let screen_width = rl_draw_handle.get_screen_width();
    let screen_height = rl_draw_handle.get_screen_height();
    for i in 0..=screen_width / tile_texture.width() {
        for j in 0..=screen_height / tile_texture.height() {
            rl_draw_handle.draw_texture(
                tile_texture,
                i * tile_texture.width(),
                j * tile_texture.height(),
                COLOR_LIGHT,
            );
        }
    }
    rl_draw_handle.draw_text(
        "CREDITS: Captainfl4me",
        screen_width - rl_draw_handle.measure_text("CREDITS: Captainfl4me", 32) - 20,
        screen_height - 40,
        32,
        COLOR_BLACK,
    );
}

pub fn draw_help_page(rl_draw_handle: &mut RaylibDrawHandle, text_to_display: Vec<&str>) {
    let screen_width = rl_draw_handle.get_screen_width();
    let screen_height = rl_draw_handle.get_screen_height();

    let panel_width: i32 = 600;
    let panel_height: i32 = 10 + 32 + 10 + 20 * (text_to_display.len() as i32) + 10;
    let panel_x = (screen_width - panel_width) / 2;
    let panel_y = (screen_height - panel_height) / 2;
    rl_draw_handle.draw_rectangle_rounded(
        Rectangle::new(
            panel_x as f32,
            panel_y as f32,
            panel_width as f32,
            panel_height as f32,
        ),
        0.1,
        20,
        COLOR_DARK,
    );

    rl_draw_handle.draw_text(
        "HELP",
        panel_x + (panel_width - rl_draw_handle.measure_text("HELP", 32)) / 2,
        panel_y + 10,
        32,
        COLOR_LIGHT,
    );
    for (i, text) in text_to_display.iter().enumerate() {
        rl_draw_handle.draw_text(
            text,
            panel_x + 10,
            panel_y + 52 + (i as i32) * 20,
            18,
            COLOR_LIGHT,
        );
    }
}
