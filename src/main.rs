use nannou::{
    prelude::*,
    noise::{Worley, Fbm, NoiseFn},
};

fn main() {
    nannou::sketch(jan_02).run()
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

fn jan_02(app: &App, frame: Frame) {
    let draw = app.draw();

    let win_rect = app.window_rect();
    draw.background().color(PLUM);

    let mx = app.mouse.x;
    let my = app.mouse.y;

    let d = Vector2::new(mx, my).distance(Vector2::new(0.0, 0.0)) / (win_rect.w().min(win_rect.h())/2.0);

    let mut r = win_rect.w() * 2.0;
    let noise = Fbm::default();
    let step = 150.0 - 20.0*d;
    let offset_strength = step/2.0;
    let mut i = 0;
    let position_jitter_magnitude = 0.001f64;
    while r > step {
        let x = mx + noise.get([(mx + r) as f64*position_jitter_magnitude, (my + r) as f64 * position_jitter_magnitude]) as f32 * offset_strength - offset_strength/2.0;
        let y = my + noise.get([(mx + r) as f64*position_jitter_magnitude, (my + r) as f64 * position_jitter_magnitude + 12340.0]) as f32 * offset_strength - offset_strength/2.0;
        draw.ellipse().xy((x,y).into()).wh((r,r).into()).hsv(
            noise.get([i as f64, i as f64]) as f32,
            noise.get([i as f64, i as f64]) as f32,
            noise.get([i as f64, i as f64]) as f32,
        );
        i += 1;
        r -= step;
    }
    draw.to_frame(app, &frame).unwrap();
}
