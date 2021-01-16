use std::collections::HashMap;
use nannou::{prelude::*, math::Matrix4, noise::{Fbm, NoiseFn}};
use nannou::rand::{rngs::SmallRng, Rng, SeedableRng};
use clingo::*;

fn main() {
    nannou::app(model).update(update).exit(exit).run();
}

struct Model {
    texture: wgpu::Texture,
    draw: nannou::Draw,
    renderer: nannou::draw::Renderer,
    noise: LoopingNoise,
    texture_capturer: wgpu::TextureCapturer,
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

    Model {
        texture,
        draw,
        renderer,
        noise: LoopingNoise::new(60*10, 10, 300),
        texture_capturer,
    }
}
#[derive(ToSymbol)]
struct Edge {
    a: i32,
    b: i32,
}
#[derive(ToSymbol)]
struct Diagonal {
    a: i32,
    b: i32,
}
#[derive(ToSymbol)]
struct Coloring(i32);
#[derive(ToSymbol)]
struct Hue(i32, u8);

fn print_model(model: &clingo::Model) {
    // retrieve the symbols in the model
    let atoms = model
        .symbols(ShowType::SHOWN)
        .expect("Failed to retrieve symbols in the model.");

    print!("Model:");

    for atom in atoms {
        // retrieve and print the symbol's string
        print!(" {}", atom.to_string().unwrap());
    }
    println!();
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let draw = &model.draw;
    draw.reset();


    let mut elapsed_frames = app.main_window().elapsed_frames();

    let window = app.main_window();
    let device = window.swap_chain_device();
    let ce_desc = wgpu::CommandEncoderDescriptor {
        label: Some("texture renderer"),
    };

    let noise = model.noise.for_frame(elapsed_frames);

    let draw = &model.draw;
    draw.reset();
    let rect = Rect::from_wh([model.texture.size()[0] as f32, model.texture.size()[1] as f32].into());

    let tile_w = rect.w() / 5.0;
    let tile_h = rect.h() / 10.0;
    let draw = &model.draw;
    draw.reset();
    let rect = Rect::from_wh([model.texture.size()[0] as f32, model.texture.size()[1] as f32].into());;


    let mut  fb = FactBase::new();
    let mut colors = vec![];
    for i in 0..75 {
        let h = noise.get(i).powf(1.6);
        fb.insert(&Hue(i as i32, (h * 256.0).min(255.0) as u8));
        colors.push(hsv(h, 1.0, 1.0));
    }

    let mut ctl = Control::new(Default::default()).expect("Failed creating Control.");
    ctl.add("base", &[], &format!("c(0..{}).", colors.len()-1)).unwrap();
    ctl.add("base", &[], "1 {coloring(X,I) : c(I)} 1 :- v(X).").unwrap();
    ctl.add("base", &[], "similar(I, J) :- |X - Y| < 40, hue(I, X), hue(J, Y).").unwrap();
    ctl.add("base", &[], ":- coloring(X,I), coloring(Y,J), edge(X,Y), hue(I, U), hue(J, V), similar(I, J).").unwrap();
    ctl.add("base", &[], ":- coloring(X,I), coloring(Y,J), diagonal(X,Y), hue(I, U), hue(J, V), -similar(I, J).").unwrap();
    //ctl.add("base", &[], ":- coloring(X,I), coloring(Y,I), edge(X,Y), c(I).").unwrap();
    ctl.add("base", &[], "v(1..50).").unwrap();

    for x in 0..5 {
        for y in 0..10 {
            let i = x + y * 5 + 1;
            for (dx, dy) in &[(1,0), (-1,0), (0,1), (0,-1)] {
                if x + dx >= 0 && x + dx < 5 {
                    if y + dy >= 0 && y + dy < 10 {
                        let x = x + dx;
                        let y = y + dy;
                        let j = x + y * 5 + 1;
                        fb.insert(&Edge { a:i, b:j});
                    }
                }
            }
            for (dx, dy) in &[(1,-1), (-1,1), (1,1), (-1,-1)] {
                if x + dx >= 0 && x + dx < 5 {
                    if y + dy >= 0 && y + dy < 10 {
                        let x = x + dx;
                        let y = y + dy;
                        let j = x + y * 5 + 1;
                        fb.insert(&Diagonal { a:i, b:j});
                    }
                }
            }
        }
    }
    ctl.add_facts(&fb);
    let fb = FactBase::new();
    let part = Part::new("base", &[]).unwrap();
    let parts = vec![part];
    ctl.ground(&parts).unwrap();

    let mut handle = ctl
        .solve(SolveMode::YIELD, &[])
        .expect("Failed retrieving solve handle.");
    let mut colorings = HashMap::new();
    loop {
        handle.resume().expect("Failed resume on solve handle.");
        match handle.model() {
            // print the model
            Ok(Some(model)) => {
                for atom in model.symbols(ShowType::SHOWN).unwrap() {
                    if atom.name().unwrap() == "coloring" {
                        let args = atom.arguments().unwrap();
                        let a = args.get(0).unwrap().number().unwrap();
                        let b = args.get(1).unwrap().number().unwrap();
                        println!("from model: {} {}", a, b);
                        colorings.insert(a, b as usize);
                    } else {
                        println!("{:?}", atom.name());
                    }
                }
            },
            // stop if there are no more models
            Ok(None) => break,
            Err(e) => panic!("Error: {}", e),
        }
    }

    for x in 0..5 {
        for y in 0..10 {
            let i = x + y * 5 + 1;
            let ci = colorings[&i];
            let color = colors[ci];
            draw.ellipse().w_h(tile_w, tile_h).x_y(x as f32 * tile_w - rect.w()/2.0 + tile_w/2.0, y as f32 * tile_h - rect.h()/2.0 + tile_h/2.0).color(color);
        }
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
