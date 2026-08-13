use macroquad::prelude::*;
use macroquad::rand::gen_range;

fn bearing(from: Vec2, to: Vec2) -> f32 {
    let d = to - from;
    d.x.atan2(-d.y).to_degrees().rem_euclid(360.0)
}

fn distance(from: Vec2, to: Vec2) -> f32 {
    ((from.x - to.x).powi(2) + (from.y - to.y).powi(2)).sqrt()
}

fn random_position(range: f32) -> Vec2 {
    vec2(gen_range(0.0, range), gen_range(0.0, range))
}

struct Game {
    // world sizes
    bullseye: Vec2,
    target: Vec2,
    call: Call,
    display_range: DisplayRange,

    //screen related for clicking
    bullseye_radius: u16, // in pixels, handling both bullseye seen, and target

    state: State,
    attempts: u8,
}

struct Radar {
    rect: Rect,
    range: f32,
}

struct Call {
    target_bearing: u16,
    target_distance_nm: f32,
}

enum State {
    Playing,
    Feedback(Feedback),
}

enum Feedback {
    Miss { miss_distance: f32 },
    Hit,
}

enum DisplayRange {
    Nm30,
    Nm60,
    Nm120,
    Nm240,
}

impl DisplayRange {
    fn nm(&self) -> f32 {
        match self {
            Self::Nm30 => 30.0,
            Self::Nm60 => 60.0,
            Self::Nm120 => 120.0,
            Self::Nm240 => 240.0,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            DisplayRange::Nm30 => "30 NM",
            DisplayRange::Nm60 => "60 NM",
            DisplayRange::Nm120 => "120 NM",
            DisplayRange::Nm240 => "240 NM",
        }
    }

    fn next(&self) -> Self {
        match self {
            DisplayRange::Nm30 => DisplayRange::Nm60,
            DisplayRange::Nm60 => DisplayRange::Nm120,
            DisplayRange::Nm120 => DisplayRange::Nm240,
            DisplayRange::Nm240 => DisplayRange::Nm30,
        }
    }
}

impl Game {
    fn handle_guess(&mut self, radar: &Radar) {
        let mouse = mouse_position();
        let click = radar.screen_to_world(vec2(mouse.0, mouse.1));

        self.attempts += 1;

        let miss_distance = click.distance(self.target);
        let hit_radius = radar.pixels_to_nm((self.bullseye_radius * 3) as f32);

        if miss_distance <= hit_radius {
            self.state = State::Feedback(Feedback::Hit);
        } else {
            self.state = State::Feedback(Feedback::Miss {
                miss_distance: miss_distance,
            });
        }
    }

    fn new() -> Self {
        let mut game = Self {
            bullseye: Vec2::ZERO,
            target: Vec2::ZERO,
            call: Call {
                target_bearing: 0,
                target_distance_nm: 0.0,
            },
            bullseye_radius: 10,
            display_range: DisplayRange::Nm60,
            state: State::Playing,
            attempts: 0,
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

            State::Feedback(Feedback::Miss { .. }) if self.attempts < 3 => {
                self.state = State::Playing;
            }

            State::Feedback(Feedback::Miss { .. }) => {
                self.shuffle();
            }

            State::Feedback(Feedback::Hit) => {
                self.shuffle();
            }
        }
    }

    fn draw_ui(&self) {
        draw_text(
            &format!("HSD Scope: {} nm", self.display_range.label()),
            20.0,
            30.0,
            24.0,
            WHITE,
        );

        //bullseye call
        draw_text(
            &format!(
                "Bullseye {:03} {:.1}",
                self.call.target_bearing, self.call.target_distance_nm
            ),
            20.0,
            60.0,
            24.0,
            WHITE,
        );

        //Feedback
        if let State::Feedback(Feedback::Miss { miss_distance }) = &self.state {
            draw_text(
                &format!("Miss by {:.1} nm", miss_distance),
                20.0,
                90.0,
                24.0,
                RED,
            );
        }
    }

    fn draw(&self, radar: &Radar) {
        // HSD
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

        // draw target if needed
        match &self.state {
            State::Playing => {}

            State::Feedback(Feedback::Hit) => {
                self.draw_target(radar);
            }

            State::Feedback(Feedback::Miss { .. }) if self.attempts >= 3 => {
                self.draw_target(radar);
            }

            State::Feedback(Feedback::Miss { .. }) => {}
        }

        self.draw_ui();
    }

    fn draw_target(&self, radar: &Radar) {
        let p = radar.world_to_screen(self.target);

        draw_circle(p.x, p.y, self.bullseye_radius as f32, RED);
    }

    fn shuffle(&mut self) {
        self.bullseye = random_position(self.display_range.nm());
        self.target = random_position(self.display_range.nm());

        self.call = Call {
            target_bearing: bearing(self.bullseye, self.target).round() as u16,
            target_distance_nm: distance(self.bullseye, self.target),
        };
        self.attempts = 0;
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
        let radar = Radar::new(game.display_range.nm());

        game.update(&radar);

        clear_background(BLACK);
        game.draw(&radar);

        next_frame().await;
    }
}
