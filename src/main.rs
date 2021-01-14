use nannou::{prelude::*, math::Matrix4, noise::{Fbm, NoiseFn}};
use nannou::rand::{rngs::SmallRng, Rng, SeedableRng};

fn main() {
    nannou::app(model).update(update).exit(exit).run();
}

struct Model {
    texture: wgpu::Texture,
    texture2: wgpu::Texture,
    draw: nannou::Draw,
    renderer: nannou::draw::Renderer,
    renderer2: nannou::draw::Renderer,
    noise: LoopingNoise,
    texture_capturer: wgpu::TextureCapturer,
}

fn model(app: &App) -> Model {
    let texture_size = [500, 500];
    let second_texture_size = [(texture_size[0]/1) * 1, (texture_size[1]/1) * 2];

    let w_id = app
        .new_window()
        .title("nannou")
        .view(view)
        .msaa_samples(8)
        .build()
        .unwrap();
    let window = app.window(w_id).unwrap();

    let device = window.swap_chain_device();

    let sample_count = window.msaa_samples();
    let texture = wgpu::TextureBuilder::new()
        .size(texture_size)
        .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED)
        .sample_count(sample_count)
        .format(wgpu::TextureFormat::Rgba16Float)
        .build(device);
    let texture2 = wgpu::TextureBuilder::new()
        .size(second_texture_size)
        .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED)
        .sample_count(sample_count)
        .format(wgpu::TextureFormat::Rgba16Float)
        .build(device);

    let draw = nannou::Draw::new();
    let descriptor = texture.descriptor();
    let renderer =
        nannou::draw::RendererBuilder::new().build_from_texture_descriptor(device, descriptor);
    let descriptor = texture2.descriptor();
    let renderer2 =
        nannou::draw::RendererBuilder::new().build_from_texture_descriptor(device, descriptor);

   let texture_capturer = wgpu::TextureCapturer::default();
   std::fs::create_dir_all(&capture_directory(app)).unwrap();

    Model {
        texture,
        texture2,
        draw,
        renderer,
        renderer2,
        noise: LoopingNoise::new(60*10, 10, 300),
        texture_capturer,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let draw = &model.draw;
    draw.reset();

    let [w, h] = model.texture.size();
    let w = w as f32;
    let h = h as f32;

    let mut elapsed_frames = app.main_window().elapsed_frames() as i32;
    elapsed_frames += 0;
    let noise = model.noise.for_frame(elapsed_frames as u64);

    draw.background().hsv(0.0, 1.0, 0.005);

    let mut circles = vec![];
    let hue = noise.get(3) as f32;
    let poop = noise.get(5) as f32 - 0.7;
    for i in 0..90 {
        let r = (((elapsed_frames + i) as f32 + poop*200.0) % 90.0) as f32 / 90.0;
        let r = r * w;
        let v = (i % 30) as f32 / 30.0;
        circles.push((r, v));
    }
    circles.sort_by_key(|(r, _)| -(r * 1000.0) as i32);
    for (r, v) in circles {
        draw.ellipse().w_h(r, r).x_y(0.0, 0.0).hsv(hue, 1.0, v);
    }

    // green boarder
    let hue = noise.get(0) as f32;
    let mut circles = vec![];
    let poop = noise.get(6) as f32 - 0.7;
    for i in 0..180 {
        let r = (((elapsed_frames + i) as f32 + poop*200.0) % 180.0) as f32 / 180.0;
        let r = r * w;
        let v = (i % 30) as f32 / 30.0;
        circles.push((r, v));
    }
    circles.sort_by_key(|(r, _)| -(r * 1000.0) as i32);
    for (r, v) in circles {
        draw.ellipse().w_h(r * 1.4, r * 1.4).x_y(w, 0.0).hsv(hue, 1.0, v);
        draw.ellipse().w_h(r * 1.4, r * 1.4).x_y(-w, 0.0).hsv(hue, 1.0, v);
        draw.ellipse().w_h(r * 1.4, r * 1.4).x_y(0.0, h).hsv(hue, 1.0, v);
        draw.ellipse().w_h(r * 1.4, r * 1.4).x_y(0.0, -h).hsv(hue, 1.0, v);
    }

    let mut circles = vec![];
    let poop = noise.get(6) as f32 - 0.7;
    for i in 0..180 {
        let r = (((elapsed_frames + i) as f32 + poop*200.0) % 180.0) as f32 / 180.0;
        let r = r * w;
        let v = (i % 30) as f32 / 30.0;
        circles.push((r, v));
    }
    circles.sort_by_key(|(r, _)| -(r * 1000.0) as i32);
    let hue = noise.get(1) as f32;
    for (r, v) in circles {
        draw.ellipse().w_h(r * 1.3, r * 1.3).x_y(w, 0.0).hsv(hue, 1.0, v);
        draw.ellipse().w_h(r * 1.3, r * 1.3).x_y(-w, 0.0).hsv(hue, 1.0, v);
        draw.ellipse().w_h(r * 1.3, r * 1.3).x_y(0.0, h).hsv(hue, 1.0, v);
        draw.ellipse().w_h(r * 1.3, r * 1.3).x_y(0.0, -h).hsv(hue, 1.0, v);
    }


    /*
    let n = 6;
    for i in 0..30 {
        let d1 = noise.get(i * n + 0).max(0.0).min(1.0).powf(1.5) as f32 * w as f32 * 2.0;
        let d2 = noise.get(i * n + 5).max(0.0).min(1.0).powf(1.5) as f32 * w as f32 * 2.0;
        let d2 = d2*0.5 + d1*0.5;
        let r2 = noise.get(i * n + 1).max(0.0).min(1.0) as f32 * w as f32;
        let a = noise.get(i * n + 2) * std::f32::consts::PI * 2.0;
        let a2 = noise.get(i * n + 6) * std::f32::consts::PI * 2.0;
        let h = noise.get(i * n + 3) as f32;
        let v = noise.get(i * n + 4).powf(1.25);
        draw.rotate(a2).ellipse().w_h(d1, d2).x_y(a.cos()*r2, a.sin()*r2).hsv(h, 1.0, v);
    }
    */

    let window = app.main_window();
    let device = window.swap_chain_device();
    let ce_desc = wgpu::CommandEncoderDescriptor {
        label: Some("texture renderer"),
    };
    let mut encoder = device.create_command_encoder(&ce_desc);
    model
        .renderer
        .render_to_texture(device, &mut encoder, draw, &model.texture);

    window.swap_chain_queue().submit(&[encoder.finish()]);

    if cfg!(feature = "save_frames") && (elapsed_frames as u64) < model.noise.period {

        let draw = &model.draw;
        draw.reset();
        let rect = Rect::from_wh([model.texture2.size()[0] as f32, model.texture2.size()[1] as f32].into());
        draw_lattice(model, rect, draw.clone());

        let mut encoder = device.create_command_encoder(&ce_desc);
        model
            .renderer
            .render_to_texture(device, &mut encoder, draw, &model.texture2);
        let snapshot = model
                 .texture_capturer
                          .capture(device, &mut encoder, &model.texture2);
        window.swap_chain_queue().submit(&[encoder.finish()]);

        let path = capture_directory(app)
             .join(format!("{:03}", elapsed_frames))
             .with_extension("png");
         snapshot
             .read(move |result| {
                 let image = result.expect("failed to map texture memory");
                 image
                     .save(&path)
                     .expect("failed to save texture to png image");
             })
             .unwrap();
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    let window_rect = app.window_rect();

    draw_lattice(model, window_rect, draw.clone());

    draw.to_frame(app, &frame).unwrap();
}

fn draw_lattice(model: &Model, rect: Rect, draw: Draw) {
    draw.background().color(BLUE);
    let [w, h] = model.texture.size();
    let scale = 1.0;
    let hw = w as f32/(scale*2.0);
    let w = w as f32/scale;
    let hh = h as f32/(scale*2.0);
    let h = h as f32/scale;
    let mut lx = rect.left() + hw/2.0;

    while lx < rect.right() + w {
        let mut ly = rect.bottom() + hh/2.0;
        while ly < rect.top() + h {
            let inner_draw = draw.translate((lx, ly).into());
            inner_draw.texture(&model.texture).w_h(hw, hh);
            let mut a = std::f32::consts::PI/2.0;
            let inner_draw = draw.translate((lx + hw, ly).into()).rotate(a);
            inner_draw.texture(&model.texture).w_h(hw, hh);
            a += std::f32::consts::PI/2.0;
            let inner_draw = draw.translate((lx + hw, ly + hh).into()).rotate(a);
            inner_draw.texture(&model.texture).w_h(hw, hh);
            a += std::f32::consts::PI/2.0;
            let inner_draw = draw.translate((lx, ly + hh).into()).rotate(a);
            inner_draw.texture(&model.texture).w_h(hw, hh);
            ly += h;
        }
        lx += w;
    }
}

fn exit(app: &App, model: Model) {
    println!("Waiting for PNG writing to complete...");
    let window = app.main_window();
    let device = window.swap_chain_device();
    model
        .texture_capturer
        .await_active_snapshots(&device)
        .unwrap();
    println!("Done!");
}


struct LoopingNoise {
    period: u64,
    streams: u32,
    samples: Vec<Vec<f32>>,
}
impl LoopingNoise {
    pub fn new(period: u64, samples: u32, streams: u32) -> Self {
        let mut rng = SmallRng::from_seed([1; 16]);
        Self {
            period,
            streams,
            samples: (0..samples).map(|_| (0..streams).map(|_| rng.gen()).collect()).collect()
        }
    }

    pub fn for_frame(&self, frame: u64) -> FrameNoise {
        let mut values = vec![0.0; self.streams as usize];
        let mut weight = 0.0;
        let t = (frame % self.period) as f32 / self.period as f32;
        let p = t * self.samples.len() as f32;
        for (i, streams) in self.samples.iter().enumerate() {
            let w = (p-i as f32).abs() / self.samples.len() as f32;
            let w2 = (p-i as f32 + self.samples.len() as f32).abs() / self.samples.len() as f32;
            let w3 = (p-i as f32 - self.samples.len() as f32).abs() / self.samples.len() as f32;
            let w = 1.0/(w2.min(w).min(w3).powf(1.5) + 0.001);
            weight += w;
            for (j, v) in streams.iter().enumerate() {
                values[j] += *v * w;
            }
        }

        FrameNoise(values.into_iter().map(|v| v / weight).collect())
    }
}

struct FrameNoise(Vec<f32>);
impl FrameNoise {
    pub fn get(&self, sample: usize) -> f32 {
        self.0[sample]
    }
}
fn capture_directory(app: &App) -> std::path::PathBuf {
    app.project_path()
        .expect("could not locate project_path")
        .join(app.exe_name().unwrap())
}
