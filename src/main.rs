use nannou::{
    rand::prelude::*,
    prelude::*,
    noise::{Worley, Fbm, NoiseFn},
    math::{Matrix4, Rad},
};

fn main() {
    nannou::sketch(jan_06).run()
}

fn captured_frame_path(app: &App, num: u64) -> std::path::PathBuf {
    app.project_path()
        .expect("failed to locate `project_path`")
        .join(app.exe_name().unwrap())
        .join(format!("{:03}", num))
        .with_extension("png")
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

fn jan_05(app: &App, frame: Frame) {
    let draw = app.draw();

    let mut rng = SmallRng::seed_from_u64(app.elapsed_frames());

    let win_rect = app.window_rect();

    draw.rect().wh((win_rect.w(), win_rect.h()).into()).color(hsva(0.1, 0.1, 0.005, 0.05));

    let noise = Fbm::default();


    let mut color = hsv(1.0, 1.0, 1.0);
    let mut loc = pt2(0.0, 0.0);
    let n_sample = (noise.get([app.time as f64, app.time as f64]) + 0.2).min(1.0).powf(4.0) as f32;
    let step_scale = 20.0 + 40.0 * n_sample;
    let h_scale = 10.0;
    let s_scale = 0.001;
    let v_scale = 0.01 * n_sample;
    for _ in 0..5000 {
        loc.x += (rng.gen::<f32>() - 0.5) * step_scale;
        loc.y += (rng.gen::<f32>() - 0.5) * step_scale;

        color.hue += (rng.gen::<f32>() - 0.5) * h_scale;
        /*
        color.saturation += (rng.gen::<f32>() - 0.5) * s_scale;
        color.saturation %= 1.0;
        */
        color.value += (rng.gen::<f32>() - 0.5) * v_scale;
        color.value %= 1.0;
        draw.ellipse().wh((15.0, 15.0).into()).xy(loc).color(color);
    }

    draw.to_frame(app, &frame).unwrap();
}

fn jan_06(app: &App, frame: Frame) {
    let draw = app.draw();
    draw.background().color(hsv(0.1, 0.1, 0.005));

    let mut rng = SmallRng::seed_from_u64(app.elapsed_frames());

    let period = 60*3;
    let noise_a = ((app.elapsed_frames() % period) as f64) / period as f64;
    let noise_a = noise_a * std::f64::consts::PI*2.0;
    let noise_scale = 0.05;
    let noise_x = noise_a.cos() * noise_scale;
    let noise_y = noise_a.sin() * noise_scale;

    let win_rect = app.window_rect();


    let lattice_w = 50.0;
    let lattice_h = 100.0;

    let mut lx = win_rect.left();
    let mut a = noise_a as f32;
    let mut reflection = 1.0;

    let noise = Fbm::default();
    let colors:Vec<_> = (0..8).map(|i| {
        hsv(
            noise.get([noise_x, noise_y + i as f64 * 10000.0]) as f32 % 1.0,
            1.0,
            noise.get([noise_x + i as f64 * 10000.0, noise_y + i as f64 * 10000.0]) as f32 % 1.0,
        )
    }).collect();
    let object = |draw: Draw| {
        for i in 0..40 {
            let r = noise.get([noise_x + 10000.0, noise_y + i as f64 * 10000.0]) as f32;
            let r2 = noise.get([noise_x + 10000.0, noise_y + i as f64 * 40000.0]) as f32;
            let x = noise.get([noise_x, noise_y + i as f64 * 10000.0]) as f32 * 3.0;
            let y = noise.get([noise_x + i as f64*1000.0, noise_y + i as f64 * 100000.0]) as f32 * 3.0;
            draw.ellipse().x_y(x,y).wh((r, r2).into()).color(colors[i % colors.len()]);
        }
    };

    while lx < win_rect.right() {
        let mut ly = win_rect.bottom();
        while ly < win_rect.top() {
            let translation = Matrix4::from_translation((lx, ly, 0.0).into());
            let rotation = Matrix4::from_angle_z(Rad(a));
            let scale = Matrix4::from_nonuniform_scale(lattice_w, lattice_h, 1.0);
            let reflectionm = Matrix4::new(
                reflection, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            );
            let draw = draw.transform(translation*scale*reflectionm*rotation);
            object(draw);
            ly += lattice_h;
            a += std::f32::consts::PI/2.0;
            reflection *= -1.0;
        }
        lx += lattice_w;
    }


    if app.elapsed_frames() >= period && app.elapsed_frames() <= period * 2{
        let file_path = captured_frame_path(app, app.elapsed_frames() - period);
        app.main_window().capture_frame(file_path);
    }
    draw.to_frame(app, &frame).unwrap();
}
