use std::collections::HashMap;
use rayon::prelude::*;

use nannou::{prelude::*, math::Matrix4, noise::{Fbm, NoiseFn}};
use nannou::rand::{rngs::SmallRng, Rng, SeedableRng, prelude::*};

mod ca;

fn main() {
    nannou::app(model).update(update).exit(exit).run();
}

struct Model {
    texture: wgpu::Texture,
    draw: nannou::Draw,
    renderer: nannou::draw::Renderer,
    noise: LoopingNoise,
    texture_capturer: wgpu::TextureCapturer,
    model: HashMap<(usize, usize), f32>,
    dots: Vec<(f32, f32, f32, f32, usize)>,
    colors: Vec<Hsv>,
}

fn model(app: &App) -> Model {
    let texture_size = [500, 800];

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

    let draw = nannou::Draw::new();
    let descriptor = texture.descriptor();
    let renderer =
        nannou::draw::RendererBuilder::new().build_from_texture_descriptor(device, descriptor);

   let texture_capturer = wgpu::TextureCapturer::default();
   std::fs::create_dir_all(&capture_directory(app)).unwrap();

   let mut rng = thread_rng();

   let mut model = HashMap::new();
   let color_count = 10;
   for a in 0..color_count {
       for b in 0..color_count {
           model.insert((a, b), (rng.gen::<f32>()-0.5) * 2.0);
       }
   }
   let dots:Vec<_> = (0..1000).map(|_| (rng.gen_range(0.0, 500.0), rng.gen_range(0.0, 800.0), 0.0, 0.0, rng.gen_range(0, color_count))).collect();



   let mut colors:Vec<_> = (0..color_count).map(|i| hsv(i as f32 / color_count as f32, 1.0, 1.0 - i as f32 / (color_count + 1) as f32)).collect();
   //colors.shuffle(&mut rng);
   println!("{:?}", colors);

    Model {
        texture,
        draw,
        renderer,
        noise: LoopingNoise::new(60*10, 10, 300),
        texture_capturer,
        model: model,
        dots,
        colors,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let draw = &model.draw;
    draw.reset();


    let mut elapsed_frames = app.main_window().elapsed_frames();
    let mut forces = Vec::with_capacity(model.dots.len());
    for (i, (x, y, _, _, c)) in model.dots.iter().enumerate() {
        let mut fx = 0.0;
        let mut fy = 0.0;
        for (j, (xx, yy, _, _, cc)) in model.dots.iter().enumerate() {
            if xx == x && yy == y {
                continue
            }
            let dx = xx-x;
            let dy = yy-y;

            let d = (dx*dx + dy*dy).sqrt();
            let du = (dx*dx + (dy+800.0).powf(2.0)).sqrt();
            let dd = (dx*dx + (dy-800.0).powf(2.0)).sqrt();
            let dl = ((dx - 500.0).powf(2.0) + dy*dy).sqrt();
            let dr = ((dx + 500.0).powf(2.0) + dy*dy).sqrt();

            let d = d.min(du).min(dd).min(dl).min(dr);

            let m = model.model[&(*c, *cc)] / d;
            fx += (dx/d) * m * 40.0;
            fy += (dy/d) * m * 40.0;
        }
        forces.push((fx, fy));
    }

    let window = app.main_window();
    let device = window.swap_chain_device();
    let ce_desc = wgpu::CommandEncoderDescriptor {
        label: Some("texture renderer"),
    };

    let noise = model.noise.for_frame(elapsed_frames);

    let draw = &model.draw;
    draw.reset();
    let rect = Rect::from_wh([model.texture.size()[0] as f32, model.texture.size()[1] as f32].into());
    draw.rect().w_h(rect.w(), rect.h()).rgba(0.1, 0.1, 0.1, 0.1);


    for ((fx, fy), (xx, yy, vx, vy, c)) in forces.into_iter().zip(&mut model.dots) {
        *vx += fx;
        *vy += fy;
        *vx = (*vx * 0.9).min(10.0).max(-10.0);
        *vy = (*vy * 0.9).min(10.0).max(-10.0);
        *xx += *vx;
        if *xx > 500.0 {
            *xx -= 500.0;
        } else if *xx < 0.0 {
            *xx += 500.0;
        }
        *yy += *vy;
        if *yy > 800.0 {
            *yy -= 800.0;
        } else if *yy < 0.0 {
            *yy += 800.0;
        }
        draw.ellipse().w_h(5.0, 5.0).x_y(*xx - 250.0, *yy - 400.0).color(model.colors[*c]);
    }


    let mut encoder = device.create_command_encoder(&ce_desc);
    model
        .renderer
        .render_to_texture(device, &mut encoder, draw, &model.texture);
    window.swap_chain_queue().submit(&[encoder.finish()]);
    if cfg!(feature = "save_frames") && elapsed_frames < model.noise.period {
        let mut encoder = device.create_command_encoder(&ce_desc);
        let snapshot = model
                 .texture_capturer
                          .capture(device, &mut encoder, &model.texture);
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

    draw.texture(&model.texture);

    draw.to_frame(app, &frame).unwrap();
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
