use sfml::graphics::{Color, CustomShape, RenderTarget, RenderWindow, ShapeImpl, Transformable};
use sfml::window::{Key, VideoMode, event, window_style, ContextSettings};
use sfml::system::{Vector2f, Vector2u};
use std::thread::sleep;
use std::time::Duration;
use geom::{Figure, Path};
use track;
use polygon::Polygon;
use polyshape::{Polyshape, Polyshapable};

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
    scale: Vector2f,
    pos: Vector2f,
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

pub fn run() {
    let mut settings = ContextSettings::default();
    settings.0.antialiasing_level=16;//(16);
    let mut window = RenderWindow::new(VideoMode::new_init(1600, 1200, 32),
                                       "Custom shape",
                                       window_style::CLOSE,
                                       //&Default::default())
                                       &settings)
                         .unwrap();
    window.set_vertical_sync_enabled(true);
    //window.set_size(&Vector2u::new(400, 300));

    let ws = window.get_size();
    println!("Window size: {:?}", ws);
    let view = View::new(Rect::new(0.0, 0.0, ws.y as f32, ws.y as f32),
                         Rect::new(-120.0, 120.0, 120.0, -120.0));
    
    /*
    let ws = window.get_size();
    println!("Window size: {:?}", ws);
    let view = View::new(Rect::new(0.0, 0.0, ws.y as f32, ws.y as f32),
                         Rect::new(-120.0, 120.0, 120.0, -120.0));
    println!("View: {:?}", view);
    let clover = track::clover(2.0, 10.0);
    let mut shape1 = CustomShape::new(Box::new(clover.paths[0].clone())).unwrap();
    let mut shape2 = CustomShape::new(Box::new(clover.paths[1].clone())).unwrap();
    //shape1.set_position(&Vector2f{x: 800.0, y: 600.0});
    //shape1.set_scale2f(5.0, -5.0);
    shape1.set_position(&view.pos());
    shape1.set_scale(&view.scale());
    shape1.set_fill_color(&Color::transparent());
    shape1.set_outline_color(&Color::red());
    shape1.set_outline_thickness(0.5);

    //shape2.set_position(&Vector2f{x: 800.0, y: 600.0});
    //shape2.set_scale2f(5.0, 5.0);
    shape2.set_position(&view.pos());
    shape2.set_scale(&view.scale());
    shape2.set_fill_color(&Color::transparent());
    shape2.set_outline_color(&Color::green());
    shape2.set_outline_thickness(0.5);
    */
    
    let mut pg = Polygon::new();
    
    let loop_cycles = 1000;
    let mut all_cycles = 0;
    
    loop {
        pg.run(loop_cycles);
        all_cycles += loop_cycles;
        println!("{}", all_cycles); 
        for event in window.events() {
            match event {
                event::Closed => return,
                event::KeyPressed { code: Key::Escape, .. } => return,
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
        window.display();
        sleep(Duration::from_millis(1));
    }
}
