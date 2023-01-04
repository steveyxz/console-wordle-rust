use std::{io::Write, process::exit};

use console::*;
use console_wordle_rust::{self, GameState, GameWorldState};
use rand::Rng;

fn main() {
    //Load in word bank and the list
    //The word bank denotes everything which can be used as a target word
    //The word list denotes everything which can be guessed by the player
    let word_bank = include_str!("word_bank.txt");
    let word_list = include_str!("word_list.txt");

    //Current world state, stores some game data
    //TODO save this data?
    let mut world_state = GameWorldState {
        current_attempts: 0,
        current_wins: 0,
        current_losses: 0,
        current_winstreak: 0,
        is_saveable: true,
    };
    let terminal = Term::stdout();

    //Styles for text, based upon console api
    let main_text_style: Style = Style::new().green();
    let main_error_style: Style = Style::new().red();
    let success_style: Style = Style::new().color256(40).bold();

    print_with_style("Welcome to Wordle!", &success_style, &terminal);
    print_with_style(
        "First, let's setup some basic settings!",
        &main_text_style,
        &terminal,
    );

    //Loop until the player has inputed a valid number for the round count, round count defaults
    //as 6, correlating to the real wordle.
    let mut has_inputed_max_attempts = false;
    let mut max_attempts: usize = 0;
    while !has_inputed_max_attempts {
        print_with_style(
            "How many attempts would you like per round? (number 3-20)",
            &main_text_style,
            &terminal,
        );
        let input: String = terminal.read_line().unwrap();
        max_attempts = match input.parse::<usize>() {
            Ok(result) => {
                if result < 3 || result > 20 {
                    print_with_style(
                        "Make sure to enter a number between 3-20",
                        &main_error_style,
                        &terminal,
                    );
                } else {
                    has_inputed_max_attempts = true;
                }
                result
            }
            Err(_) => {
                if input == "" {
                    has_inputed_max_attempts = true;
                } else {
                    print_with_style(
                        "Make sure to enter a valid number!",
                        &main_error_style,
                        &terminal,
                    );
                }
                6
            }
        }
    }

    loop {
        print_with_style(
            "Do you want to save/load your stats? (y/n)",
            &main_text_style,
            &terminal,
        );
        let input: String = terminal.read_line().unwrap();
        if input == "y" {
            max_attempts = 6;
            world_state.load();
            print_with_style("Loaded data!", &success_style, &terminal);
            print_with_style(
                format_args!("{:?}", world_state).to_string().as_str(),
                &success_style,
                &terminal,
            );
            world_state.is_saveable = true;
            break;
        } else if input == "n" {
            print_with_style(
                "Your stats were not loaded, and will not save",
                &main_text_style,
                &terminal,
            );
            world_state.is_saveable = false;
            break;
        }
    }

    //Ask for a username kinda thing
    print_with_style(
        "What should we refer to you as?",
        &main_text_style,
        &terminal,
    );
    let name = terminal.read_line().unwrap();

    //Start wordle, create a game instance
    print_with_style(
        ("Ok ".to_owned() + &name + &"! Your wordle will now begin!".to_owned()).as_str(),
        &main_text_style,
        &terminal,
    );

    let mut current_game = GameState {
        target_word: get_random_target_word(word_bank),
        max_guesses: max_attempts,
        current_guesses: 0,
        guessed_words: Vec::new(),
    };

    loop {
        loop {
            //Get player to input a guess, process the guess
            print_with_style("Type in a guess! ", &main_text_style, &terminal);
            let guess = terminal.read_line().unwrap().trim().to_lowercase();
            //Check if the word is correct
            if current_game.target_word == guess {
                print_with_style("You got the word correct!", &success_style, &terminal);
                current_game.guess(&guess);
                terminal
                    .write_line(current_game.get_board().as_str())
                    .expect("Error while writing board to terminal!");
                world_state.current_attempts += 1;
                world_state.current_wins += 1;
                world_state.current_winstreak += 1;
                break;
            }
            //Check if the player has guessed the word already
            if current_game.guessed_words.contains(&guess) {
                print_with_style(
                    "You have already guessed that word!",
                    &main_error_style,
                    &terminal,
                );
                terminal
                    .write_line(current_game.get_board().as_str())
                    .expect("Error while writing board to terminal!");
                continue;
            }
            //Check if word is valid, if the word is valid,
            //it means that the guess count of the player should
            //decrease by one
            if is_valid(&guess, word_list) {
                print_with_style(
                    "You didn't get the word correct :(",
                    &main_error_style,
                    &terminal,
                );
                let is_over = current_game.guess(&guess);
                terminal
                    .write_line(current_game.get_board().as_str())
                    .expect("Error while writing board to terminal!");
                if is_over {
                    print_with_style(
                        "You ran out of guesses! Game over D:",
                        &main_error_style,
                        &terminal,
                    );
                    world_state.current_attempts += 1;
                    world_state.current_losses += 1;
                    world_state.current_winstreak = 0;
                    break;
                }
            } else {
                print_with_style("Invalid word!", &main_error_style, &terminal);
                terminal
                    .write_line(current_game.get_board().as_str())
                    .expect("Error while writing board to terminal!");
            }
        }
        world_state.save();
        loop {
            print_with_style(
                "Do you want to try again? (y/n)",
                &main_text_style,
                &terminal,
            );
            let input: String = terminal.read_line().unwrap();
            if input == "y" {
                current_game = GameState {
                    target_word: get_random_target_word(word_bank),
                    max_guesses: max_attempts,
                    current_guesses: 0,
                    guessed_words: Vec::new(),
                };
                print_with_style("Starting new game!", &success_style, &terminal);
                print_with_style(
                    format_args!("{:?}", world_state).to_string().as_str(),
                    &success_style,
                    &terminal,
                );
                break;
            } else if input == "n" {
                print_with_style("Thanks for playing!", &success_style, &terminal);
                exit(0)
            }
        }
    }
}

fn get_random_target_word(word_bank: &str) -> String {
    //Get a random word from the word bank use rand api
    let seperated_words: Vec<&str> = word_bank.lines().collect();
    let mut rng = rand::thread_rng();
    let random_number = rng.gen_range(0..seperated_words.len());
    seperated_words.get(random_number).unwrap().to_string()
}

fn print_with_style(content: &str, style: &Style, mut terminal: &Term) {
    //Use console api to print with a style
    terminal
        .write_fmt(format_args!("{}\n", style.apply_to(content)))
        .expect("Writing to console failed with an error!");
}

fn is_valid(guess: &String, word_list: &str) -> bool {
    //If guess isn't of length 5 cannot be valid
    if guess.len() != 5 {
        false;
    }
    //Now check if the word is inside the list of valid words
    let words: Vec<&str> = word_list.lines().collect();
    return words.contains(&guess.as_str());
}
