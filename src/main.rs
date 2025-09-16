use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    style::{self, Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
    ExecutableCommand,
};
use rand::Rng;
use std::collections::VecDeque;
use std::io::{stdout, Result};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Game {
    snake: VecDeque<Position>,
    direction: Direction,
    next_direction: Direction,
    food: Position,
    score: u32,
    game_over: bool,
    width: i32,
    height: i32,
    speed: Duration,
}

impl Game {
    fn new(width: i32, height: i32) -> Self {
        let mut snake = VecDeque::new();
        let start_pos = Position {
            x: width / 2,
            y: height / 2,
        };
        snake.push_back(start_pos);
        snake.push_back(Position {
            x: start_pos.x - 1,
            y: start_pos.y,
        });
        snake.push_back(Position {
            x: start_pos.x - 2,
            y: start_pos.y,
        });

        let mut game = Game {
            snake,
            direction: Direction::Right,
            next_direction: Direction::Right,
            food: Position { x: 0, y: 0 },
            score: 0,
            game_over: false,
            width,
            height,
            speed: Duration::from_millis(150),
        };
        game.spawn_food();
        game
    }

    fn spawn_food(&mut self) {
        let mut rng = rand::thread_rng();
        loop {
            let pos = Position {
                x: rng.gen_range(0..self.width),
                y: rng.gen_range(0..self.height),
            };
            if !self.snake.contains(&pos) {
                self.food = pos;
                break;
            }
        }
    }

    fn update(&mut self) {
        if self.game_over {
            return;
        }

        self.direction = self.next_direction;

        let head = *self.snake.front().unwrap();
        let new_head = match self.direction {
            Direction::Up => Position {
                x: head.x,
                y: head.y - 1,
            },
            Direction::Down => Position {
                x: head.x,
                y: head.y + 1,
            },
            Direction::Left => Position {
                x: head.x - 1,
                y: head.y,
            },
            Direction::Right => Position {
                x: head.x + 1,
                y: head.y,
            },
        };

        if new_head.x < 0
            || new_head.x >= self.width
            || new_head.y < 0
            || new_head.y >= self.height
            || self.snake.contains(&new_head)
        {
            self.game_over = true;
            return;
        }

        self.snake.push_front(new_head);

        if new_head == self.food {
            self.score += 10;
            self.spawn_food();
            if self.speed.as_millis() > 80 {
                self.speed = Duration::from_millis(self.speed.as_millis() as u64 - 2);
            }
        } else {
            self.snake.pop_back();
        }
    }

    fn handle_input(&mut self, key: KeyCode) {
        let new_dir = match key {
            KeyCode::Up if self.direction != Direction::Down => Direction::Up,
            KeyCode::Down if self.direction != Direction::Up => Direction::Down,
            KeyCode::Left if self.direction != Direction::Right => Direction::Left,
            KeyCode::Right if self.direction != Direction::Left => Direction::Right,
            _ => return,
        };
        self.next_direction = new_dir;
    }

    fn render(&self) -> Result<()> {
        let mut stdout = stdout();
        stdout.execute(cursor::Hide)?;
        stdout.execute(cursor::MoveTo(0, 0))?;

        // Üst çerçeve
        stdout.execute(SetForegroundColor(Color::Cyan))?;
        stdout.execute(Print("┌"))?;
        for _ in 0..self.width * 2 {
            stdout.execute(Print("─"))?;
        }
        stdout.execute(Print("┐\r\n"))?;

        for y in 0..self.height {
            // Sol çerçeve
            stdout.execute(SetForegroundColor(Color::Cyan))?;
            stdout.execute(Print("│"))?;

            for x in 0..self.width {
                let pos = Position { x, y };
                if self.snake.front() == Some(&pos) {
                    stdout.execute(SetForegroundColor(Color::Green))?;
                    stdout.execute(Print("●"))?;
                } else if self.snake.contains(&pos) {
                    stdout.execute(SetForegroundColor(Color::DarkGreen))?;
                    stdout.execute(Print("●"))?;
                } else if pos == self.food {
                    stdout.execute(SetForegroundColor(Color::Red))?;
                    stdout.execute(Print("●"))?;
                } else {
                    stdout.execute(Print(" "))?;
                }

                if x < self.width - 1 {
                    stdout.execute(Print(" "))?;
                } else {
                    stdout.execute(Print(" "))?;
                }
            }

            // Sağ çerçeve
            stdout.execute(SetForegroundColor(Color::Cyan))?;
            stdout.execute(Print("│\r\n"))?;
        }

        // Alt çerçeve
        stdout.execute(SetForegroundColor(Color::Cyan))?;
        stdout.execute(Print("└"))?;
        for _ in 0..self.width * 2 {
            stdout.execute(Print("─"))?;
        }
        stdout.execute(Print("┘\r\n"))?;

        stdout.execute(SetBackgroundColor(Color::Reset))?;
        stdout.execute(SetForegroundColor(Color::White))?;
        stdout.execute(cursor::MoveTo(0, self.height as u16 + 2))?;
        stdout.execute(Print(format!("Score: {}  ", self.score)))?;

        if self.game_over {
            stdout.execute(cursor::MoveTo(0, self.height as u16 + 4))?;
            stdout.execute(SetForegroundColor(Color::Red))?;
            stdout.execute(Print("GAME OVER! Press 'r' to restart or 'q' to quit"))?;
        } else {
            stdout.execute(cursor::MoveTo(0, self.height as u16 + 4))?;
            stdout.execute(Print("Use arrow keys to move, 'q' to quit"))?;
        }

        stdout.execute(style::ResetColor)?;
        Ok(())
    }
}

fn main() -> Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.execute(terminal::Clear(ClearType::All))?;
    stdout.execute(cursor::Hide)?;

    let (width, height) = (30, 20);
    let mut game = Game::new(width, height);
    let mut last_update = Instant::now();

    loop {
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('r') if game.game_over => {
                        game = Game::new(width, height);
                    }
                    arrow_key if !game.game_over => game.handle_input(arrow_key),
                    _ => {}
                }
            }
        }

        if last_update.elapsed() >= game.speed {
            game.update();
            last_update = Instant::now();
        }

        game.render()?;
    }

    stdout.execute(cursor::Show)?;
    stdout.execute(terminal::Clear(ClearType::All))?;
    terminal::disable_raw_mode()?;

    Ok(())
}