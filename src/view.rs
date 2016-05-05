use sfml::graphics::{Color, RenderTarget, RenderWindow, ShapeImpl, Text, Transformable, Font};
use sfml::window::{Key, VideoMode, event, window_style, ContextSettings};
use sfml::system::{Vector2f, Vector2i};
use std::thread::sleep;
use std::time::Duration;
use geom::{Figure, Path};
use track;
use polygon::Polygon;
use polyshape::{Polyshape, Polyshapable};
use std::fs;
use std::path;

#[derive(Clone, Copy)]
pub struct TriangleShape;

impl ShapeImpl for TriangleShape {
    fn get_point_count(&self) -> u32 {
        3
    }

    fn get_point(&self, point: u32) -> Vector2f {
        match point {
            0 => Vector2f { x: 20., y: 580. },
            1 => Vector2f { x: 400., y: 20. },
            2 => Vector2f { x: 780., y: 580. },
            p => panic!("Non-existent point: {}", p),
        }
    }
}

impl ShapeImpl for Path {
    fn get_point_count(&self) -> u32 {
        self.sects.len() as u32
    }

    fn get_point(&self, point: u32) -> Vector2f {
        let pt = self.sects[point as usize].p0;
        Vector2f {x: pt.x as f32, y: pt.y as f32}
    }
}

#[derive(Clone, Copy)]
pub struct Rect<T> {
    pub left: T,
    pub top: T,
    pub right: T,
    pub bottom: T,
}

impl<T> Rect<T> {
    pub fn new(left: T, top: T, right: T, bottom: T) -> Rect<T> {
        Rect {
            left: left,
            top: top,
            right: right,
            bottom: bottom
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct View {
    pub scale: Vector2f,
    pub pos: Vector2f,
}

impl View {
    pub fn new(window_rect: Rect<f32>, object_rect: Rect<f32>) -> View {
        let ww = window_rect.right - window_rect.left;
        let wh = window_rect.bottom - window_rect.top;

        let ow = object_rect.right - object_rect.left;
        let oh = object_rect.bottom - object_rect.top;

        let px = window_rect.left - object_rect.left * ww / ow;
        let py = window_rect.top - object_rect.top * wh / oh;
        View {
            scale: Vector2f::new(ww / ow, wh / oh),
            pos: Vector2f::new(px, py)
        }
    }
    
    pub fn scale(&self) -> Vector2f {
        self.scale
    }
    
    pub fn pos(&self) -> Vector2f {
        self.pos
    }
}

//pub struct Screen {
//    pub shapes: Vec<CustomShape>
//}

fn dir_of_workspace(workspace: &str) -> path::PathBuf {
    let dir = path::Path::new("./workspaces/").join(workspace);
    fs::create_dir_all(dir.as_path());
    dir
}

pub fn run(workspace: &str) {
    let ws_dir = dir_of_workspace(workspace);
    let mut settings = ContextSettings::default();
    settings.0.antialiasing_level = 16;
    let mut window = RenderWindow::new(VideoMode::new_init(1920, 970, 32),
                                       "Polygon",
                                       window_style::CLOSE,
                                       //&Default::default())
                                       &settings)
                         .unwrap();
    window.set_position(&Vector2i::new(0, 0));
    window.set_vertical_sync_enabled(true);
    //window.set_size(&Vector2u::new(400, 300));

    let ws = window.get_size();
    println!("Window size: {:?}", ws);
    let mut view = View::new(Rect::new(0.0, 0.0, ws.y as f32, ws.y as f32),
                         Rect::new(-120.0, 120.0, 120.0, -120.0));
        
    let mut pg = Polygon::new(ws_dir.clone());
    
    let loop_cycles = 1000;
    let mut all_cycles = 0;
    let font_filename = "C:\\Users\\super\\.cargo\\registry\\src\\github.com-88ac128001ac3a9a\\sfml-0.11.2\\examples\\resources\\sansation.ttf";
    //let font_filename = "/Users/aovchinn/Downloads/SourceCodePro_FontsOnly-1.017/TTF/SourceCodePro-Regular.ttf";    
    let font = Font::new_from_file(font_filename).unwrap();
    
    let mut pause = false;
    
    loop {
        if pause {
            sleep(Duration::from_millis(100));
        } else {
            pg.run(loop_cycles);
            all_cycles += loop_cycles;
        }
        //if all_cycles % (100 * loop_cycles) == 0 {
        //    pg.save(ws_dir);
        //}
        for event in window.events() {
            match event {
                event::Closed => return,
                event::KeyPressed { code: Key::Escape, .. } => return,
                event::KeyPressed { code: Key::A, ..} => { 
                    view.scale.x *= 2.0;
                    view.scale.y *= 2.0;
                },
                event::KeyPressed { code: Key::Z, ..} => { 
                    view.scale.x /= 2.0;
                    view.scale.y /= 2.0; 
                },
                event::KeyPressed { code: Key::Left, ..} => { 
                    view.pos.x += 50.0;
                },
                event::KeyPressed { code: Key::Right, ..} => { 
                    view.pos.x -= 50.0;
                },
                event::KeyPressed { code: Key::Up, ..} => { 
                    view.pos.y += 50.0;
                },
                event::KeyPressed { code: Key::Down, ..} => { 
                    view.pos.y -= 50.0;
                },
                event::KeyPressed { code: Key::P, ..} => { 
                    pg.learner.borrow().print();
                },
                event::KeyPressed { code: Key::Space, ..} => { 
                    pause = !pause;
                },                
                _ => {}
            }
        }

        window.clear(&Color::white());
        //window.draw(&shape1);
        //window.draw(&shape2);
        let world = pg.world.borrow();
        let ps = world.get_polyshape(view);
        window.draw(&ps);
        let car = pg.car.borrow();
        let ps_car = car.get_polyshape(view);
        window.draw(&ps_car);
        
        let text = format!("Cycles: {}\nSpeed:  {}\nWheels: {}\nAct[0]: {}\n\
                            Act[1]: {}\nReward: {}\nX: {}\nY: {}",
                    all_cycles, car.speed, car.wheels_angle,
                    world.last_action[0], world.last_action[1],
                    pg.last_reward, car.center.x, car.center.y);
        
        let mut txt = Text::new().unwrap();
        txt.set_font(&font);
        txt.set_character_size(24);
        txt.set_string(&text);
        txt.set_position2f(1200.0, 30.0);
        txt.set_color(&Color::black());
        window.draw(&txt);
        
        window.display();
        //sleep(Duration::from_millis(1));
    }
}
