use rand::{thread_rng, Rng};

#[derive(Debug)]
pub struct BirdState {
    pub y: f32,
    pub dy: f32,
    pub score: u32,
    pub pipes: Vec<f32>,
    pub time: u32,
}
impl Default for BirdState {
    fn default() -> Self {
        BirdState {
            y: 0.5,
            dy: 0.0,
            score: 0,
            pipes: vec![],
            time: 0,
        }
    }
}

const screent: u32 = 600;
const pipet: u32 = 40;
const pipes: u32 = 4;
const birdt: u32 = 60;

impl BirdState {
    pub fn update(mut self, jump: bool) -> Result<Self, u32> {
        self.y += self.dy / 60.0;

        // gravity
        self.dy -= 0.01;
        // jump
        if jump {
            self.dy = 0.2
        }

        // spawn pipes in if necessary
        if self.time % (screent / pipes) == 0 {
            self.pipes.push(rand::thread_rng().gen_range(-1.0..=1.0));
            if self.pipes.len() >= pipes as usize {
                self.pipes.remove(0);
            }
        }

        self.time += 1;
        Ok(self)
    }
}
