use nannou::{
    prelude::*,
    noise::{Worley, NoiseFn},
};

fn main() {
    nannou::sketch(jan_01).run()
}

fn jan_01(app: &App, frame: Frame) {
    let draw = app.draw();

    let win_rect = app.window_rect();
    draw.background().color(PLUM);

    let tile_count_x = 200;
    let tile_count_y = 20;
    let tile_width = win_rect.w() / tile_count_x as f32;
    let tile_height = win_rect.h() / tile_count_y as f32;

    let mx = app.mouse.x;
    let my = app.mouse.y;

    let size = vec2(tile_width as f32, tile_height as f32);
    let r = nannou::geom::Rect::from_wh(size)
        .align_left_of(win_rect)
        .align_top_of(win_rect);

    let noise = Worley::default();
    for grid_x in 0..tile_count_x+1 {
        for grid_y in 0..tile_count_y+1 {
            let r = r
                .shift_x(tile_width * grid_x as f32)
                .shift_y(-tile_height * grid_y as f32);
            let dx = r.x() - mx;
            let dy = r.y() - my;
            let d = (dx*dx + dy*dy).sqrt() / 80.0;
            let a = 1.5;
            let b = -2.4;
            let x = (a + b*d) * d.cos();
            let y = (a + b*d) * d.sin();
            draw.rect().xy(r.xy()).wh(r.wh()).hsv(
                noise.get([x as f64, y as f64]) as f32,
                noise.get([x as f64, y as f64]) as f32,
                noise.get([x as f64, y as f64]) as f32,
            );
        }
    }
    draw.to_frame(app, &frame).unwrap();
}
