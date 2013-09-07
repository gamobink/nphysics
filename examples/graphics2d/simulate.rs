use std::os;
use rsfml::graphics::render_window::*;
use rsfml::window::video_mode::*;
use rsfml::window::context_settings::*;
use rsfml::window::event;
use rsfml::window::keyboard;
use rsfml::graphics::color::*;
use nalgebra::vec::Vec2;
use nphysics::aliases::dim2;
use camera::Camera;
use fps::Fps;
use engine::GraphicsManager;
use draw_helper;

fn usage(exe_name: &str) {
    println("Usage: " + exe_name);
    println("The following keyboard commands are supported:");
    println("    t     - pause/continue the simulation.");
    println("    s     - pause then execute only one simulation step.");
    println("    space - display/hide contacts.");
    println("    CTRL + click + drag - select and drag an object using a ball-in-socket joint.");
}

pub fn simulate(builder: &fn(&mut GraphicsManager) -> dim2::BodyWorld2d<f64>) {
    let args = os::args();

    if args.len() > 1 {
        usage(args[0]);
        os::set_exit_status(1);
        return;
    }

    let running    = @mut Running;
    let draw_colls = @mut false;

    let mode    = VideoMode::new_init(800, 600, 32);
    let setting = ContextSettings{
        depthBits:         10,
        stencilBits:       10,
        antialiasingLevel: 2,
        majorVersion:      0,
        minorVersion:      1
    };
    let mut rwindow =
        match RenderWindow::new(mode, ~"nphysics demo", sfDefaultStyle, &setting) {
            Some(rwindow) => rwindow,
            None => fail!(~"Error on creating window")
        };

    let mut camera = Camera::new();

    rwindow.set_framerate_limit(60);


    let mut graphics = GraphicsManager::new();
    let mut physics  = builder(&mut graphics);
    let mut fps      = Fps::new();
    let mut cursor_pos;
    let grabbed_object: Option<@mut dim2::Body2d<f64>> = None;
    let grabbed_object_joint: Option<@mut dim2::BallInSocket2d<f64>> = None;

    while rwindow.is_open() {
        loop {
            match rwindow.poll_event() {
                event::KeyPressed(code, _, _, _, _) => {
                    match code {
                        keyboard::Escape => rwindow.close(),
                        keyboard::S      => *running = Step,
                        keyboard::Space  => *draw_colls = !*draw_colls,
                        keyboard::T      => {
                            if *running == Stop {
                                *running = Running;
                            }
                            else {
                                *running = Stop;
                            }
                        },
                        _                => { }
                    }
                },
                ev @ event::MouseMoved(x, y) => {
                    cursor_pos = Vec2::new(x as f64, y as f64);
                    match grabbed_object {
                        Some(_) => {
                            let joint = grabbed_object_joint.unwrap();
                            joint.set_local2(cursor_pos);
                        },
                        None => camera.handle_event(&ev)
                    };
                },
                event::Closed  => rwindow.close(),
                event::NoEvent => break,
                e              => camera.handle_event(&e)
            }
        }

        rwindow.clear(~Color::black());

        fps.reset();

        if *running != Stop {
            physics.step(0.016);
        }

        if *running == Step {
            *running = Stop;
        }
        fps.register_delta();
        graphics.draw(&mut rwindow, &camera);

        camera.activate_scene(&mut rwindow);
        if *draw_colls {
            draw_helper::draw_colls(&rwindow, &mut physics);
        }

        camera.activate_ui(&mut rwindow);
        fps.draw_registered(&mut rwindow);

        rwindow.display();
    }

    rwindow.close();
}

#[deriving(Eq)]
enum RunMode {
    Running,
    Stop,
    Step
}