use std::time::Duration;
use nannou::prelude::*;
use nannou_osc as osc;

const SEND_PORT: u16 = 9000;
const RECV_PORT: u16 = 8000;

fn main () {
    nannou::app(model)
        .update(update)
        .run();
}

enum Mode {
    Reset,
    Running,
    Pause,
    Ended,
}

struct Model {
    _window: window::Id,
    red_score: u8,
    blue_score: u8,
    time_left: Duration,
    mode: Mode,
    sender: osc::Sender<osc::Connected>,
    receiver: osc::Receiver,
}

fn model (app: &App) -> Model {
    let _window = app.new_window()
        .fullscreen()
        .view(view)
        .key_pressed(key_pressed)
        .build()
        .unwrap();
    let time_left = Duration::new(120, 0);
    let target_address = format!("127.0.0.1:{}", SEND_PORT);
    let sender = osc::sender()
        .expect("Could not bind to default socket.")
        .connect(target_address.clone())
        .expect(format!("Could not connect to socket at address: {}", target_address).as_str());
    let receiver = osc::receiver(RECV_PORT).unwrap();
    Model {
        _window,
        red_score: 0,
        blue_score: 0,
        time_left,
        mode: Mode::Reset,
        sender,
        receiver,
    }
}


fn update (_app: &App, model: &mut Model, update: Update) {
    for packet in model.receiver.try_iter() {
        println!("Packet from {}: {:?}", packet.1.ip(), packet.0);
        for message in packet.0.into_msgs() {
            match message.addr.as_str() {
                "/red/add" => {
                    if message.args.len() > 0 {
                        let score = match message.args[0].clone().int() {
                            Some(s) => (s & 0xf) as u8,
                            None => 0u8,
                        };
                        if model.red_score + score < 10 {
                            model.red_score += score;
                        }
                    }
                },
                "/red/sub" => {
                    if message.args.len() > 0 {
                        let score = match message.args[0].clone().int(){
                            Some(s) => (s & 0xf) as u8,
                            None => 0u8,
                        };
                        if model.red_score >= score {
                            model.red_score -= score;
                        }
                    }
                },
                "/blue/add" => {
                    if message.args.len() > 0 {
                        let score = match message.args[0].clone().int(){
                            Some(s) => (s & 0xf) as u8,
                            None => 0u8,
                        };
                        if model.blue_score + score < 10 {
                            model.blue_score += score;
                        }
                    }
                },
                "/blue/sub" => {
                    if message.args.len() > 0 {
                        let score = match message.args[0].clone().int(){
                            Some(s) => (s & 0xf) as u8,
                            None => 0u8,
                        };
                        if model.blue_score >= score {
                            model.blue_score -= score;
                        }
                    }
                },
                "/reset" => {
                    model.mode = Mode::Reset;
                    model.red_score = 0;
                    model.blue_score = 0;
                },
                "/pause" => {
                    model.mode = match model.mode {
                        Mode::Reset => Mode::Running,
                        Mode::Running => Mode::Pause,
                        Mode::Pause => Mode::Running,
                        Mode::Ended => {
                            model.red_score = 0;
                            model.blue_score = 0;
                            Mode::Reset
                        },
                    }
                },
                _ => println!("Unknown message address: {}", message.addr),
            }
        }

        //TODO: Log incomming message sources, and try to connect to them for sending scores and time.
    }


    match model.mode {
        Mode::Reset => model.time_left = Duration::new(120, 0),
        Mode::Running => model.time_left = model.time_left.checked_sub(update.since_last).or_else(|| {Some(Duration::ZERO)}).unwrap(),
        Mode::Pause => {},
        Mode::Ended => model.time_left = Duration::ZERO,
    }
    if model.time_left <= Duration::ZERO {
        model.mode = Mode::Ended;
    }
}

fn key_pressed (app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Q => if model.red_score < 8 { model.red_score += 2 },
        Key::A => if model.red_score > 1 { model.red_score -= 2 },
        Key::W => if model.red_score < 8 { model.red_score += 1 },
        Key::S => if model.red_score > 0 { model.red_score -= 1 },
        Key::T => if model.blue_score < 8 { model.blue_score += 2 },
        Key::G => if model.blue_score > 1 { model.blue_score -= 2 },
        Key::R => if model.blue_score < 8 { model.blue_score += 1 },
        Key::F => if model.blue_score > 0 { model.blue_score -= 1 },
        Key::Space => {
            model.mode = match model.mode {
                Mode::Reset => Mode::Running,
                Mode::Running => Mode::Pause,
                Mode::Pause => Mode::Running,
                Mode::Ended => {
                    model.red_score = 0;
                    model.blue_score = 0;
                    Mode::Reset
                },
            }
        },
        Key::Back => {
            model.mode = Mode::Reset;
            model.red_score = 0;
            model.blue_score = 0;
        },
        Key::Escape => app.quit(),
        _ => {},
    }
}

fn view (app: &App, model: &Model, frame: Frame) {
    //frame.clear(PURPLE);
    let size = frame.rect();
    let draw = app.draw();
    let font_size = match (size.h() / 6.0).to_u32() {
        Some(u) => u,
        None => 16,
    };
    let time_size = match (size.h() / 5.0).to_u32() {
        Some(u) => u,
        None => 18,
    };

    // Sides
    draw.background().color(BLACK);
    draw.rect().color(RED).w_h((size.w() - 10.0) / 2.0, size.h() - 10.0).x_y(-size.w() / 4.0, 0.0);
    draw.rect().color(BLUE).w_h((size.w() - 10.0) / 2.0, size.h() - 10.0).x_y(size.w() / 4.0, 0.0);

    // Score area
    draw.rect().color(BLACK).w_h(size.w() / 2.0, size.h() / 5.0).x_y(0.0, size.h() / 4.0);
    draw.ellipse().color(BLACK).w_h(size.h() / 5.0, size.h() / 5.0).x_y(-size.w() / 4.0, size.h() / 4.0);
    draw.ellipse().color(BLACK).w_h(size.h() / 5.0, size.h() / 5.0).x_y(size.w() / 4.0, size.h() / 4.0);

    // Scores
    draw.rect().color(WHITE).w_h(size.w() / 20.0, size.h() / 40.0).x_y(0.0, size.h() / 4.0);
    draw.text(model.red_score.to_string().as_str()).font_size( font_size ).color(WHITE).w_h(size.w() * 0.3, size.h() / 6.0).x_y(-size.w() * 0.18, size.h() * 0.28 );
    draw.text(model.blue_score.to_string().as_str()).font_size( font_size ).color(WHITE).w_h(size.w() * 0.3, size.h() / 6.0).x_y(size.w() * 0.18, size.h() * 0.28 );

    // Time area
    draw.rect().color(BLACK).w_h(size.w() / 3.0, size.h() / 4.0).x_y(0.0, -size.h() / 4.0);
    draw.ellipse().color(BLACK).w_h(size.h() / 4.0, size.h() / 4.0).x_y(-size.w() / 6.0, -size.h() / 4.0);
    draw.ellipse().color(BLACK).w_h(size.h() / 4.0, size.h() / 4.0).x_y(size.w() / 6.0, -size.h() / 4.0);

    // Time
    let minutes = model.time_left.mins().floor();
    let seconds = model.time_left.secs().floor() - (minutes * 60.0);
    draw.text(format!("{}:{:02}", minutes, seconds).as_str()).font_size( time_size ).color(WHITE).w_h(size.w() * 0.5, size.h() / 5.0).x_y(0.0, -size.h() * 0.22 );

    let address = "/match/time";
    let data = vec![osc::Type::Float(minutes as f32), osc::Type::Float(seconds as f32)];
    let packet = (address, data);
    model.sender.send(packet).ok();

    // Key explanation
    draw.text("Q +2    W +1\nW -2    S -1\nSpace (Start/Pause)").color(WHITE).x_y(-size.w() / 4.0, -(size.h() - 70.0) / 2.0);
    draw.text("R +1    T +2\nF -1    G -2\nBackspace (Reset)").color(WHITE).x_y(size.w() / 4.0, -(size.h() - 70.0) / 2.0);

    draw.to_frame(app, &frame).unwrap();
}
