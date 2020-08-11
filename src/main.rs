extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate piston_window;
extern crate gfx_device_gl;
extern crate time;
extern crate rand;
extern crate image as im;

mod world;

use world::*;

use piston_window::*;
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use gfx_device_gl::{Factory, Resources, CommandBuffer};

use std::{time::Instant};
use rand::Rng;
use noise::{NoiseFn, Perlin};
use im::{ImageBuffer};



pub struct App {
    rotation: f64,  // Rotation for the square.
    dt: f64,
    world: World,
    viewport_offsets: Location,
    image_buffer: im::ImageBuffer<im::Rgba<u8>, Vec<u8>>,
    texture_context: TextureContext<Factory, Resources, CommandBuffer>,
    texture: Option<Texture<Resources>>
}

impl App {

    pub fn new(World: World, window: &mut PistonWindow) -> Self {
        let mut app = App {
            rotation: 0.0,
            dt: 0.0,
            world: World,
            viewport_offsets: Location{
                x:0.0,
                y:0.0
            },
            image_buffer: ImageBuffer::new(window.size().width as u32, window.size().height as u32),
            texture_context: gfx_graphics::TextureContext {
                factory: window.factory.clone(),
                encoder: window.factory.create_command_buffer().into()
            },
            texture: None
        };
        app.texture = Some(Texture::from_image(
            &mut app.texture_context,
            &app.image_buffer,
            &TextureSettings::new()
        ).unwrap());
        app
    }

    fn render(&mut self, window: &mut PistonWindow, event: &impl GenericEvent) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

        let square = rectangle::square(0.0, 0.0, 50.0);
        let rotation = self.rotation;

        let width = window.size().width;
        let height = window.size().height;

        let (x, y) = (width / 2.0, height / 2.0);

        let world = &self.world;
        let viewport_offsets = &self.viewport_offsets;

        self.image_buffer = ImageBuffer::new(window.size().width as u32, height as u32);
        let perlin = Perlin::new();
        for x in 0..window.size().width as u32 {
            let val = perlin.get([((x as f64)+viewport_offsets.x)/1024.0 ,((x + 10) as f64)/1024.0]);
            let mut scaled = val * (height / 2.0) + (height / 2.0);
            println!("Scaled: {}", scaled);
            if scaled > height {
                scaled = height - 1.0;
            }
            let color = (scaled/height * 255.0) as u8;
            self.image_buffer.put_pixel(x,(scaled) as u32, im::Rgba([color,color,color,255]));
            for y in 0..scaled as u32 { // Fill in under the flow
                self.image_buffer.put_pixel(x,y as u32, im::Rgba([color,color,color,255]));
            }
        }

        let texture: &mut Texture<Resources>;
        let texture_context = &mut self.texture_context;
        match &mut self.texture {
            Some(txt) => texture = txt,
            None => panic!("Texture is null")
        }
        let update = texture.update(texture_context, &self.image_buffer);
        if update.is_err() {
            return;
        }
        window.draw_2d(event, |c, g, device| {
            texture_context.encoder.flush(device);
            clear(WHITE, g);
            
            /*for chunk in &world.chunks{
                for block in &chunk.blocks {
                        let transform = c.transform.
                        trans(viewport_offsets.x, viewport_offsets.y)
                        .trans(x,y)
                        .trans(block.location.x*50f64,block.location.y*50f64);
                    rectangle(block.color, square, transform, g);
                }
            }*/

            let transform = c.transform.
            rot_deg(180.0)
            .trans(-width, -height);
            image(texture, transform, g);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
    }
}

const MOVESPEED:f64 = 20.0;
const WINDOW_WIDTH:f64 = 450.0;
const WINDOW_HEIGHT:f64 = 300.0;

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: PistonWindow = WindowSettings::new("Definitely not LIAN", [WINDOW_WIDTH, WINDOW_HEIGHT])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let example_world = create_example_world();
    let mut app = App::new(example_world, &mut window);

    let mut delta = Instant::now();
    let mut fps_time = Instant::now();
    let mut frame_time = 0;

    let mut pressed_buttons: Vec<Key> = vec![];

    let event_settings = EventSettings::new();
    let mut events = Events::new(event_settings);

    while let Some(e) = events.next(&mut window) {
        app.dt = delta.elapsed().as_secs_f64();
        if(fps_time.elapsed().as_millis() >= 1000) {
            println!("FPS: {}", frame_time as f64 / fps_time.elapsed().as_secs_f64());
            fps_time = Instant::now();
            frame_time = 0;
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            if !pressed_buttons.contains(&key) {
                pressed_buttons.push(key);
            }
                
        };
        if let Some(Button::Keyboard(key)) = e.release_args() {
            for i in 0..pressed_buttons.len() {
                let elem = pressed_buttons[i];
                if elem.code() == key.code() {
                    pressed_buttons.remove(i);
                    break;
                }
            }
        };

        for key in pressed_buttons.iter() {
            match key {
                Key::W | Key::Up => app.viewport_offsets.y += MOVESPEED,
                Key::A | Key::Left => app.viewport_offsets.x += MOVESPEED,
                Key::S | Key::Down => app.viewport_offsets.y -= MOVESPEED,
                Key::D | Key::Right => app.viewport_offsets.x -= MOVESPEED,
                _ => ()
            }
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }

        if let Some(_args) = e.render_args() {
            app.render(&mut window, &e);
        }
        delta = Instant::now();
        frame_time += 1;
    }
}

fn create_example_world() -> World {
    let mut world = World {
        chunks: Vec::new()
    };
    let mut chunk = Chunk {
        blocks: Vec::new()
    };
    
    let mut rng = rand::thread_rng();
    for x in -10..10 {
        for y in -10..10 {
            let block = Block {
                location: Location{
                    x:x.into(),
                    y:y.into()
                },
                color: [rng.gen(), rng.gen(), rng.gen(), 1.0]
            };
            chunk.blocks.push(block);
        }
    }
    world.chunks = Vec::new();
    world.chunks.push(chunk);
    return world;
}