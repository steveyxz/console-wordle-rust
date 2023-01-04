#[macro_use]
extern crate json;

use std::fmt::Display;
use std::path::Path;
use std::{self, fs};

use console::Style;

pub struct GameWorldState {
    pub current_attempts: i32,
    pub current_wins: i32,
    pub current_losses: i32,
    pub current_winstreak: i32,
    pub is_saveable: bool,
}

impl GameWorldState {
    pub fn load(&mut self) {
        if !Path::new("data.json").exists() {
            fs::write("data.json", "{}").expect("Error while reading data!");
            self.save();
            return;
        }
        let data = json::parse(fs::read_to_string("data.json").unwrap().as_str()).unwrap();
        self.current_attempts = data["current_attempts"].as_i32().unwrap();
        self.current_wins = data["current_wins"].as_i32().unwrap();
        self.current_losses = data["current_losses"].as_i32().unwrap();
        self.current_winstreak = data["current_winstreak"].as_i32().unwrap();
    }

    pub fn save(&self) {
        if !self.is_saveable {
            return;
        }

        let data = object! {
            current_attempts: self.current_attempts,
            current_wins: self.current_wins,
            current_losses: self.current_losses,
            current_winstreak: self.current_winstreak,
        };

        fs::write("data.json", data.dump()).expect("Error while writing data!");
    }
}

impl std::fmt::Debug for GameWorldState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Current game stats:\nAttempts: {}\nWins: {}\nLosses: {}\nWinstreak: {}",
            self.current_attempts, self.current_wins, self.current_losses, self.current_winstreak
        )
    }
}

impl Display for GameWorldState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct GameState {
    pub target_word: String,
    pub max_guesses: usize,
    pub current_guesses: usize,
    pub guessed_words: Vec<String>,
}

impl GameState {
    //Assumes word is valid already -> won't do a validility check!
    //Returns true if the game is over, otherwise false
    pub fn guess(&mut self, word: &String) -> bool {
        self.guessed_words.push(word.to_string());
        self.current_guesses += 1;
        if self.is_over() {
            return true;
        }
        return false;
    }

    pub fn is_over(&self) -> bool {
        self.current_guesses >= self.max_guesses
    }

    pub fn get_board(&self) -> String {
        let mut board = String::from("▁▁▁▁▁\n");
        let empty = String::from("@@@@@");
        for i in 0..self.max_guesses {
            if i < self.guessed_words.len() {
                board.push_str(
                    self.get_compatibility(self.guessed_words.get(i).unwrap().to_string())
                        .as_str(),
                )
            } else {
                board.push_str(empty.as_str());
            }
            board.push_str("\n");
        }
        board.push_str("▔▔▔▔▔");
        return board;
    }

    //Gets the wordle output of some other word in comparison to the current word
    pub fn get_compatibility(&self, other: String) -> String {
        //Loop through each character of the other word
        //If the character is not in the target word, append gray character
        //If the character is in the target word,
        //.- If the current character is equal to the character at that index in the target word,
        //   the character will be green no matter what.
        // - Otherwise,
        //   - Count the total number of that character in the target word
        //   - If that total number in the other word is more than the target,
        //     - Find difference n between the two character totals
        //     - Loop through the other word until the current character, counting all other instances of the character
        //     - If the total number of instances is lower than n, then the character is yellow
        //     - Otherwise the character will be gray
        //   - Otherwise, the letter is yellow

        let gray_style = Style::new();
        let green_style = Style::new().green();
        let yellow_style = Style::new().yellow();

        let mut result = String::from("");

        let mut counter = 0;

        for c in other.chars() {
            if self.target_word.contains(c) {
                if c == self.target_word.chars().nth(counter).unwrap() {
                    //Green character
                    apply(&mut result, &green_style, c);
                } else {
                    let count_in_target = count(&self.target_word, c);
                    let count_in_other = count(&other, c);

                    if count_in_other > count_in_target {
                        //n will be the number of excess characters within the guessed word
                        let n = count_in_other - count_in_target;
                        let mut character_counter = 0;
                        for character_index in 0..counter {
                            if other.chars().nth(character_index).unwrap() == c {
                                character_counter += 1;
                            }
                        }
                        if character_counter < n {
                            //Yellow character
                            apply(&mut result, &yellow_style, c);
                        } else {
                            //Gray character
                            apply(&mut result, &gray_style, c);
                        }
                    } else {
                        //Yellow character
                        apply(&mut result, &yellow_style, c);
                    }
                }
            } else {
                //Gray character
                apply(&mut result, &gray_style, c);
            }
            counter += 1;
        }

        return result;
    }
}

fn apply(result: &mut String, style: &Style, character: char) {
    result.push_str(
        format_args!("{}", style.apply_to(character))
            .to_string()
            .as_str(),
    )
}

fn count(string: &String, character: char) -> usize {
    let mut count = 0;
    string.chars().for_each(|i| {
        if i == character {
            count += 1;
        }
    });
    return count;
}
