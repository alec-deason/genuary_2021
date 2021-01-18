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
    ca_model: ca::Model,
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

   let mut models = vec![];
   for _ in 0..20 {
       let rule_count = rng.gen_range(3, 10);
       let state_count = rng.gen_range(2,6);
       models.push(ca::Model::new_random(25, 40, rule_count, state_count, &mut rng));
   }
   let mut best_score = 0.0;
   let mut best_model = models[0].clone();
   for _ in 0..20 {
       let stats:Vec<_> = models.par_iter_mut().map(|m| {
           m.step_for(500);
           let stats = m.stats();
           stats.path_score
       }).collect();
       let mut new_models:Vec<_> = models.drain(..).zip(stats.into_iter()).collect();
       new_models.sort_by_key(|(_, s)| (s * 100.0) as i32);
       new_models.reverse();
       println!("best of generation: {}", new_models[0].1);
       if new_models[0].1 > best_score {
           best_score = new_models[0].1;
           best_model = new_models[0].0.clone();
       }
       /*
       if new_models[0].1 < 5.0 {
           chosen_model = Some(new_models[0].0.clone());
           break
       }
       */
       let count = new_models.len()/4;
       models.extend(new_models.into_iter().take(count).map(|(mut m, _)| { m.reset_random(&mut rng); m}));
       for _ in 0..3 {
           let mut m = best_model.clone();
            m.mutate(&mut rng);
            models.push(m);
       }
       for i in 0..models.len() {
           let mut m = models[i].clone();
            m.mutate(&mut rng);
            models.push(m);
       }
       while models.len() < 8 {
           let rule_count = rng.gen_range(3, 10);
           let state_count = rng.gen_range(2,6);
           models.push(ca::Model::new_random(25, 40, rule_count, state_count, &mut rng));
       }
   }
   println!("chosen model: {} {}", best_model.rule_count(), best_model.state_count());
   best_model.reset();

   let mut colors:Vec<_> = (0..best_model.state_count()).map(|i| hsv(i as f32 / best_model.state_count() as f32, 1.0, 1.0 - i as f32 / best_model.state_count() as f32)).collect();
   colors.shuffle(&mut rng);
   println!("{:?}", colors);

    Model {
        texture,
        draw,
        renderer,
        noise: LoopingNoise::new(60*10, 10, 300),
        texture_capturer,
        ca_model: best_model,
        colors,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let draw = &model.draw;
    draw.reset();


    let mut elapsed_frames = app.main_window().elapsed_frames();
    model.ca_model.step();

    let window = app.main_window();
    let device = window.swap_chain_device();
    let ce_desc = wgpu::CommandEncoderDescriptor {
        label: Some("texture renderer"),
    };

    let noise = model.noise.for_frame(elapsed_frames);

    let draw = &model.draw;
    draw.reset();
    let rect = Rect::from_wh([model.texture.size()[0] as f32, model.texture.size()[1] as f32].into());


    for (i, s) in model.ca_model.states().iter().enumerate() {
        let x = i % 25;
        let y = i / 25;
        draw.rect().w_h(20.0, 20.0).x_y(x as f32 * 20.0 - 250.0 + 10.0, y as f32 * 20.0 - 400.0 + 10.0).color(hsv(0.0, 1.0, 0.01));
        let mut color = model.colors[*s as usize];
        draw.rect().w_h(16.0, 16.0).x_y(x as f32 * 20.0 - 250.0 + 10.0, y as f32 * 20.0 - 400.0 + 10.0).color(color);
        color.hue += 180.0;
        color.value = 0.2;
        draw.rect().w_h(10.0, 10.0).x_y(x as f32 * 20.0 - 250.0 + 10.0, y as f32 * 20.0 - 400.0 + 10.0).color(color);
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
