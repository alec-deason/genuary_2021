use nannou::{
    prelude::*,
    noise::{Worley, Fbm, NoiseFn},
};

fn main() {
    nannou::sketch(jan_04).run()
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

fn jan_03(app: &App, frame: Frame) {
    let draw = app.draw();

    let win_rect = app.window_rect();
    draw.background().color(rgb(0.05, 0.05, 0.05));
    let noise = Fbm::default();
    for i in 0..5000 {
        let x = noise.get([i as f64, i as f64]) as f32 * win_rect.w() * 10.0;
        let y = noise.get([i as f64 + 12345.0, i as f64]) as f32 * win_rect.h() * 10.0;
        let v = noise.get([app.time as f64, app.time as f64 + i as f64 * 1000.0]).abs().powf(1.0) as f32 ;
        draw.ellipse().xy((x,y).into()).wh((4.0, 4.0).into()).hsv(0.0, 0.0, 1.0-v);
    }
    let noise_scale = 0.001;

    let h = 238.9/360.0;
    let s = 0.935;
    let mut v = 0.80;

    for i in 0..10 {
        let b = 0.8 - noise.get([i as f64, i as f64]) as f32*0.6;
        let o = noise.get([i as f64, i as f64]) as f32 * 10.0 + if i%2==0 { app.time } else { -app.time };
        let y = win_rect.bottom() + win_rect.h() * (1.0-i as f32 / 10.0) - 100.0;
        let mut vertices:Vec<_> = (0..win_rect.w() as u32).map(|x| {

            let p = 100.0 - 10.0 * noise.get([x as f64 * noise_scale, y as f64]) as f32;
            let y = y + (x as f32 / p + o).sin() * 30.0;
            pt2(x as f32 - win_rect.w()/2.0, y)
        }).collect();
        let mut cap_vertices = vertices.clone();
        let mut cap_vertices_lower:Vec<_> = vertices.iter().copied().map(|p| pt2(p.x, (p.y-y)*0.8+y - 40.0)).collect();
        cap_vertices_lower.reverse();
        cap_vertices.extend(cap_vertices_lower);
        vertices.push(pt2(1000.0, -1000.0));
        vertices.push(pt2(-1000.0, -1000.0));

        draw.polygon().points(vertices.clone()).hsv(h, s, v);
        draw.polygon().points(cap_vertices).hsv(h, s, v+0.1);
        draw.polyline().weight(5.0).points(vertices).hsv(h, s, v-0.1);

        v -= 0.8/10.0;
    }

    draw.to_frame(app, &frame).unwrap();
}

fn jan_04(app: &App, frame: Frame) {
    let draw = app.draw();
    draw.background().color(hsv(0.1, 0.1, 0.005));

    let win_rect = app.window_rect();

    let noise = Fbm::default();
    let noise_scale = 0.0015;
    let line_count = 20000;

    let time_dilation = 0.1;
    let time = app.time * time_dilation;

    for x in 0..line_count {
        let x = (x as f32 / line_count as f32) * win_rect.w() - win_rect.w()/3.0;
        let y = (x + time).sin() * ((50.0 + 100.0 * noise.get([(x as f64 + time as f64), x as f64]) as f32) + 200.0 * ((x + time) * 0.01).sin());
        let len = 100.0 + 300.0 * noise.get([(x as f64 + time as f64) * noise_scale, y as f64 * noise_scale]) as f32;
        let slope = (x + time).cos();
        let xa = x - len;
        let ya = y - len*slope;
        let xb = x + len;
        let yb = y + len*slope;

        let weight = 1.0 + 5.0 * noise.get([(x as f64 + time as f64), x as f64]) as f32;
        let h = (noise.get([(x as f64 + time as f64), x as f64]) as f32 + time * 0.01) % 1.0;
        draw.line().start(pt2(xa, ya)).end(pt2(xb, yb)).hsv(h, 1.0, len/200.0).weight(weight);
    }
    draw.to_frame(app, &frame).unwrap();
}
