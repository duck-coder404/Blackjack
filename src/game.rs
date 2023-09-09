use std::time::Duration;
use std::thread;

use crate::Configuration;
use crate::card::hand::{DealerHand, Hand, PlayerHand};
use crate::card::shoe::Shoe;
use crate::io::{Action, place_bet, make_move};

pub fn play(config: Configuration) {
    println!("Welcome to Blackjack!");
    let mut deck: Box<dyn Shoe> = config.decks.into();
    let mut player_chips = config.chips;
    while let Some(bet) = place_bet(player_chips, config.max_bet, config.min_bet) {
        player_chips -= bet;
        println!("You bet {bet} chips. You have {player_chips} chips remaining.");

        let mut player_hand = PlayerHand::new(deck.draw(), bet);
        let mut dealer_hand = DealerHand::new(deck.draw(), config.soft17);

        player_hand += deck.draw();
        dealer_hand += deck.draw();

        // The player may now play their hand, which may turn into multiple hands through splitting
        // (skip if dealer has blackjack)
        let mut player_hands = vec![player_hand];
        if !dealer_hand.is_21() {
            play_hands(&mut player_hands, &dealer_hand, &mut deck, &mut player_chips);
        }

        // At this point, all player hands are done and the dealer reveals their down card
        dealer_hand.reveal_down_card();

        if player_hands.iter().any(|hand| hand.is_stood()) {
            // At least one hand was played and stood on, so the dealer must finish their hand
            while !dealer_hand.is_over() {
                dealer_hand += deck.draw();
            }
        }

        // At this point, all hands are done
        // For each hand, determine the result and payout
        let dealer_status = dealer_hand.status();
        let chips_won: u32 = player_hands
            .into_iter()
            .map(|hand| hand.winnings(&dealer_status, &config.payout))
            .sum();

        pause();
        match chips_won {
            0 => println!("You lose!"),
            chips if chips < bet => println!("You make back {chips} chips."),
            chips if chips == bet => println!("You push!"),
            chips => println!("You win {chips} chips!"),
        }
        player_chips += chips_won;
        pause();
        deck.shuffle_if_needed(&config.shuffle);
    }
    println!("You finished with {player_chips} chips.");
    println!("Goodbye!");
    pause();
}

fn play_hands(hands: &mut Vec<PlayerHand>, dealer_hand: &DealerHand, deck: &mut Box<dyn Shoe>, player_chips: &mut u32) {
    while let Some(hand) = hands.iter_mut().find(|hand| !hand.is_over()) {
        pause();
        println!(
            "What would you like to do? ({} against {})",
            hand, dealer_hand.showing()
        );
        match make_move(hand.len(), hand.is_pair(), *player_chips >= hand.bet()) {
            Action::Stand => {
                println!("You stand!");
                hand.stand();
            }
            Action::Hit => {
                println!("You hit!");
                *hand += deck.draw();
            }
            Action::DoubleDown => {
                println!("You double and put another {} chips down!", hand.bet());
                *player_chips -= hand.bet(); // The player pays another equal bet
                hand.double(deck.draw());
            }
            Action::Split => {
                println!("You split your hand and put another {} chips down!", hand.bet());
                *player_chips -= hand.bet(); // The player pays another equal bet for the new hand
                let mut new_hand = PlayerHand::split(hand);
                *hand += deck.draw();
                new_hand += deck.draw();
                hands.push(new_hand);
            }
        }
    }
}

fn pause() {
    thread::sleep(Duration::from_secs(1));
}
