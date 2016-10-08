use sfml::graphics::{Color, RenderTarget, RenderWindow, RenderStates,
            ShapeImpl, Text, Transform, Transformable, Font, BlendMode};
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
use std::ops::Deref;
use plot::{Plot};

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
    let mut window = RenderWindow::new(VideoMode::new_init(1820, 970, 32),
                                       "Polygon",
                                       window_style::CLOSE,
                                       //&Default::default())
                                       &settings)
                         .unwrap();
    window.set_position(&Vector2i::new(100, 0));
    window.set_vertical_sync_enabled(true);
    //window.set_size(&Vector2u::new(400, 300));

    let ws = window.get_size();
    println!("Window size: {:?}", ws);
    let mut view = View::new(Rect::new(0.0, 0.0, ws.y as f32, ws.y as f32),
                         Rect::new(-120.0, 120.0, 120.0, -120.0));

    let plot_view = View::new(Rect::new(0.0, 0.0, 200.0, 200.0),
                              Rect::new(0.0, 1.0, 2.0, 0.0));

    let mut pg = Polygon::new(ws_dir.clone());
    let state_dim = pg.current_world().state.len() as i32;
    let mut v_fn = Vec::new();
    let mut ac_fn0 = Vec::new();
    let mut ac_fn1 = Vec::new();
    for i in 0..state_dim {
        v_fn.push(pg.v_fn(i as u32));
        ac_fn0.push(pg.ac_fn(i as u32, 0));
        ac_fn1.push(pg.ac_fn(i as u32, 1));
    }
    let mut v_n: i32 = 0;
    //return;

    let loop_cycles = 100;
    let mut all_cycles = 0;
    let font_filename = "C:\\Users\\super\\.cargo\\registry\\src\\github.com-88ac128001ac3a9a\\sfml-0.11.2\\examples\\resources\\sansation.ttf";
    //let font_filename = "/Users/aovchinn/Downloads/SourceCodePro_FontsOnly-1.017/TTF/SourceCodePro-Regular.ttf";
    let font = Font::new_from_file(font_filename).unwrap();

    let mut pause = false;

    let mut screen = 0;

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
                    pg.learner.print();
                },
                event::KeyPressed { code: Key::Space, ..} => {
                    pause = !pause;
                },
                event::KeyPressed { code: Key::Num0, ..} => {
                    screen = 0;
                },
                event::KeyPressed { code: Key::Num1, ..} => {
                    screen = 1;
                },
                event::KeyPressed { code: Key::I, ..} => {
                    v_n += 1;
                    if v_n > state_dim-1 {
                        v_n = state_dim-1;
                    }
                },
                event::KeyPressed { code: Key::U, ..} => {
                    v_n -= 1;
                    if v_n < 0 {
                        v_n = 0;
                    }
                },
                _ => {}
            }
        }

        window.clear(&Color::white());

        if screen == 0 {
            let world = pg.current_world();
            let ps = world.get_polyshape(view);
            window.draw(&ps);
            let car = &pg.current_world().car;
            let ps_car = car.get_polyshape(view);
            window.draw(&ps_car);
            let sigma = pg.learner.state.sigma.borrow();

            let text = format!("Cycles: {}\nSpeed:  {}\nWheels: {}\nAct[0]: {}\n\
                                Act[1]: {}\nReward: {}\nX: {}\nY: {}\n\
                                Offset: {}\nSigma: {}",
                        all_cycles, car.speed, car.wheels_angle,
                        world.last_action[0], world.last_action[1],
                        pg.last_reward, car.center.x, car.center.y,
                        10.0 * world.way.offset(&world.old_way_point, &world.way_point),
                        sigma.deref());

            let mut txt = Text::new().unwrap();
            txt.set_font(&font);
            txt.set_character_size(24);
            txt.set_string(&text);
            txt.set_position2f(1200.0, 30.0);
            txt.set_color(&Color::black());
            window.draw(&txt);
        } else if screen == 1 {
            //let mut plot = Plot::new(sin as fn(f64) -> f64, -10.0, 10.0, 1000);
            //(&|x: f64| x*x, -10.0, 10.0, 1000);
            let mut plot_v = Plot::new(&|x| v_fn[v_n as usize](x), -1.0, 1.0, 100);
            plot_v.set_viewport(10.0, 10.0, 600.0, 600.0);
            let mut plot_ac0 = Plot::new(&|x| ac_fn0[v_n as usize](x), -1.0, 1.0, 100);
            plot_ac0.set_viewport(620.0, 10.0, 600.0, 600.0);
            let mut plot_ac1 = Plot::new(&|x| ac_fn1[v_n as usize](x), -1.0, 1.0, 100);
            plot_ac1.set_viewport(1230.0, 10.0, 600.0, 600.0);
            let mut rs_v = RenderStates::new(BlendMode::blend_none(), plot_v.transform(), None, None);
            let mut rs_ac0 = RenderStates::new(BlendMode::blend_none(), plot_ac0.transform(), None, None);
            let mut rs_ac1 = RenderStates::new(BlendMode::blend_none(), plot_ac1.transform(), None, None);
            window.draw_with_renderstates(&plot_v, &mut rs_v);
            window.draw_with_renderstates(&plot_ac0, &mut rs_ac0);
            window.draw_with_renderstates(&plot_ac1, &mut rs_ac1);

            let text = format!("N: {}", v_n);

            let mut txt = Text::new().unwrap();
            txt.set_font(&font);
            txt.set_character_size(24);
            txt.set_string(&text);
            txt.set_position2f(0.0, 10.0);
            txt.set_color(&Color::black());
            window.draw(&txt);
        }

        window.display();
        //sleep(Duration::from_millis(1));
    }
}

fn sin(x: f64) -> f64{
    x.sin().exp().sin()
}