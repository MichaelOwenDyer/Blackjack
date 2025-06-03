use crossterm::event::KeyCode;

use crate::game::Blackjack;

#[derive(Debug, Default)]
pub struct App {
    pub games: Vec<Blackjack>,
    pub selected_game: usize,
    pub should_quit: bool,
}

impl App {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            games: Vec::new(),
            selected_game: 0,
            should_quit: false,
        }
    }

    #[must_use]
    pub fn current_game(&self) -> Option<&Blackjack> {
        self.games.get(self.selected_game)
    }

    pub fn simulate(&mut self) {
        for game in &mut self.games {
            game.simulate();
        }
    }

    pub fn tick(&mut self) {
        for game in &mut self.games {
            game.tick();
        }
    }

    pub fn input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('g') => self.add_game(),
            KeyCode::Char('q') => self.delete_game(),
            KeyCode::Up => self.cursor_up(),
            KeyCode::Down => self.cursor_down(),
            key => self.input_current_game(key),
        }
    }

    pub fn add_game(&mut self) {
        self.games.push(Blackjack::new());
        self.selected_game = self.games.len() - 1;
    }

    pub fn delete_game(&mut self) {
        if !self.games.is_empty() {
            self.games.remove(self.selected_game);
            if !self.games.is_empty() {
                self.selected_game = (self.selected_game + self.games.len() - 1) % self.games.len();
            }
        }
    }

    pub fn cursor_up(&mut self) {
        self.selected_game = (self.selected_game + self.games.len() - 1) % self.games.len();
    }

    pub fn cursor_down(&mut self) {
        self.selected_game = (self.selected_game + 1) % self.games.len();
    }

    pub fn input_current_game(&mut self, key: KeyCode) {
        if let Some(game) = self.games.get_mut(self.selected_game) {
            game.input(key);
        }
    }
}
