use macroquad::prelude::*;
use macroquad::rand::gen_range;

fn bearing(from: Vec2, to: Vec2) -> f32 {
    let d = to - from;
    d.x.atan2(-d.y).to_degrees().rem_euclid(360.0)
}

fn random_position(range: f32) -> Vec2 {
    vec2(gen_range(0.0, range), gen_range(0.0, range))
}

struct Game {
    // world sizes
    bullseye: Vec2,
    target: Vec2,
    radar_range: f32,

    //screen related for clicking
    bullseye_radius: u16, // in pixels, handling both bullseye seen, and target

    state: State,
}

struct Radar {
    rect: Rect,
    range: f32,
}

struct Hint {
    target_bearing: u16,
    target_distance_nm: f32,
    miss_distance_nm: f32,
}

enum State {
    Playing,
    Feedback(Feedback),
}

enum Feedback {
    Miss(Hint),
    Hit,
}

//let bearing = bearing(self.bullseye, self.target).round() as u16;

impl Game {
    fn handle_guess(&mut self, _radar: &Radar) {
        self.state = State::Feedback(Feedback::Hit);
    }

    fn new() -> Self {
        let mut game = Self {
            bullseye: Vec2::ZERO,
            target: Vec2::ZERO,
            bullseye_radius: 10,
            radar_range: 60.0,
            state: State::Playing,
        };
        game.shuffle();
        game
    }

    fn update(&mut self, radar: &Radar) {
        if !is_mouse_button_pressed(MouseButton::Left) {
            return;
        }

        match &self.state {
            State::Playing => self.handle_guess(radar),

            State::Feedback(Feedback::Miss(_)) => {
                self.state = State::Playing;
            }

            State::Feedback(Feedback::Hit) => {
                self.shuffle();
                self.state = State::Playing;
            }
        }
    }

    fn draw(&self, radar: &Radar) {
        // Radar outline
        draw_rectangle_lines(
            radar.rect.x,
            radar.rect.y,
            radar.rect.w,
            radar.rect.h,
            2.0,
            GREEN,
        );

        // Bullseye
        let p = radar.world_to_screen(self.bullseye);

        draw_circle(p.x, p.y, self.bullseye_radius as f32, BLUE);
    }

    fn shuffle(&mut self) {
        self.bullseye = random_position(self.radar_range);
        self.target = random_position(self.radar_range);
        self.state = State::Playing;
    }
}

impl Radar {
    fn new(range_nm: f32) -> Self {
        let size = (screen_width() - 200.0).min(screen_height());

        Self {
            rect: Rect::new(
                (screen_width() - size) / 2.0,
                (screen_height() - size) / 2.0,
                size,
                size,
            ),
            range: range_nm,
        }
    }

    fn world_to_screen(&self, pos: Vec2) -> Vec2 {
        vec2(
            self.rect.x + pos.x / self.range * self.rect.w,
            self.rect.y + pos.y / self.range * self.rect.h,
        )
    }

    fn screen_to_world(&self, pos: Vec2) -> Vec2 {
        vec2(
            (pos.x - self.rect.x) / self.rect.w * self.range,
            (pos.y - self.rect.y) / self.rect.h * self.range,
        )
    }

    fn nm_to_pixels(&self, nm: f32) -> f32 {
        nm * self.rect.w / self.range
    }

    fn pixels_to_nm(&self, pixels: f32) -> f32 {
        pixels * self.range / self.rect.w
    }
}

#[macroquad::main("Bullseye")]
async fn main() {
    let mut game = Game::new();

    loop {
        let radar = Radar::new(game.radar_range);

        game.update(&radar);

        clear_background(BLACK);
        game.draw(&radar);

        next_frame().await;
    }
}
