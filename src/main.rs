use nannou::{prelude::*, math::Matrix4, noise::{Fbm, NoiseFn}};

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    texture: wgpu::Texture,
    draw: nannou::Draw,
    renderer: nannou::draw::Renderer,
    noise: LoopingNoise,
}

fn model(app: &App) -> Model {
    let texture_size = [1500, 1500];

    let w_id = app
        .new_window()
        .title("nannou")
        .view(view)
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

    let draw = nannou::Draw::new();
    let descriptor = texture.descriptor();
    let renderer =
        nannou::draw::RendererBuilder::new().build_from_texture_descriptor(device, descriptor);

    Model {
        texture,
        draw,
        renderer,
        noise: LoopingNoise::new(60*3, 3, 60),
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let draw = &model.draw;
    draw.reset();

    let [w, h] = model.texture.size();

    let elapsed_frames = app.main_window().elapsed_frames();
    let noise = model.noise.for_frame(elapsed_frames);

    draw.background().hsv(0.0, 1.0, 0.005);
    let n = 8;
    for i in 0..3 {
        let r = noise.get(i * n + 0).max(0.0).min(1.0) as f32 * w as f32 * 0.5;
        let x = (noise.get(i * n + 1) - 0.5) * w as f32;
        let y = (noise.get(i * n + 2) - 0.5) * h as f32;
        let mut x2 = (noise.get(i * n + 3) - 0.5) * w as f32;
        let mut y2 = (noise.get(i * n + 4) - 0.5) * h as f32;
        let mut x3 = -x2;
        let mut y3 = -y2;
        if i % 2 == 0 {
            y2 = h as f32/2.0;
            y3 = -(h as f32/2.0);
        } else {
            x2 = w as f32/2.0;
            x3 = -(w as f32/2.0);
        }
        let h = noise.get(i * n + 7) as f32;
        draw.polyline().join_round().caps_round().weight(r).points(vec![
            [x2, y2],
            [x, y],
            [x3, y3],
        ]).hsv(h, 1.0, 1.0);
    }

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
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    let window_rect = app.window_rect();

    draw.background().color(BLUE);

    let [w, h] = model.texture.size();
    let mut lx = window_rect.left();

    while lx < window_rect.right() + w as f32/10.0 {
        let mut ly = window_rect.bottom();
        while ly < window_rect.top() + h as f32/10.0 {
            let inner_draw = draw.translate((lx, ly).into());
            inner_draw.texture(&model.texture).w_h(w as f32/20.0, h as f32/20.0);
            let mut a = std::f32::consts::PI/2.0;
            let inner_draw = draw.translate((lx + w as f32/20.0, ly).into()).rotate(a);
            inner_draw.texture(&model.texture).w_h(w as f32/20.0, h as f32/20.0);
            a += std::f32::consts::PI/2.0;
            let inner_draw = draw.translate((lx + w as f32/20.0, ly + h as f32/20.0).into()).rotate(a);
            inner_draw.texture(&model.texture).w_h(w as f32/20.0, h as f32/20.0);
            a += std::f32::consts::PI/2.0;
            let inner_draw = draw.translate((lx, ly + h as f32/20.0).into()).rotate(a);
            inner_draw.texture(&model.texture).w_h(w as f32/20.0, h as f32/20.0);
            ly += h as f32/10.0;
        }
        lx += w as f32/10.0;
    }
    draw.to_frame(app, &frame).unwrap();
}


struct LoopingNoise {
    period: u64,
    streams: u32,
    samples: Vec<Vec<f32>>,
}
impl LoopingNoise {
    pub fn new(period: u64, samples: u32, streams: u32) -> Self {
        Self {
            period,
            streams,
            samples: (0..samples).map(|_| (0..streams).map(|_| random_f32()).collect()).collect()
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
