use std::thread;
use std::time::Duration;

use crate::card::hand::{DealerHand, Hand, PlayerHand};
use crate::card::dispenser::CardDispenser;
use crate::io::{make_move, offer_early_surrender, place_bet, Action};
use crate::{Configuration, Surrender};

pub fn play(config: Configuration) {
    println!("Welcome to Blackjack!");
    let mut dispenser: CardDispenser = config.decks.into();
    let mut player_chips = config.chips;
    while let Some(bet) = place_bet(player_chips, config.max_bet, config.min_bet) {
        player_chips -= bet;
        println!("You bet {bet} chips. You have {player_chips} chips remaining.");

        let mut player_hand = PlayerHand::new(dispenser.draw_card(), bet);
        let mut dealer_hand = DealerHand::new(dispenser.draw_card(), config.soft17);

        player_hand += dispenser.draw_card();
        dealer_hand += dispenser.draw_card();

        if dealer_hand.showing() >= 10 {
            if (config.surrender == Surrender::Early || config.surrender == Surrender::Both)
                && offer_early_surrender()
            {
                println!("You surrender!");
                player_hand.surrender();
            }
            pause();
            println!("The dealer checks their hand for blackjack...");
        }

        // The player may now play their hand, which may turn into multiple hands through splitting
        // (skip if dealer has blackjack)
        let mut player_hands = vec![player_hand];
        if !dealer_hand.is_21() {
            play_hands(
                &mut player_hands,
                &dealer_hand,
                &mut dispenser,
                &mut player_chips,
                &config.surrender,
            );
        }

        // At this point, all player hands are done and the dealer reveals their down card
        dealer_hand.reveal_down_card();

        if player_hands.iter().any(|hand| hand.is_stood()) {
            // At least one hand was played and stood on, so the dealer must finish their hand
            while !dealer_hand.is_over() {
                dealer_hand += dispenser.draw_card();
            }
        }

        // At this point, all hands are done
        // For each hand, determine the result and payout
        let dealer_status = dealer_hand.status();
        let chips_won: u32 = player_hands
            .into_iter()
            .map(|hand| hand.winnings(&dealer_status, &config.blackjack_payout))
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
        dispenser.shuffle_if_needed(config.penetration);
    }
    println!("You finished with {player_chips} chips.");
    println!("Goodbye!");
    pause();
}

fn play_hands(
    hands: &mut Vec<PlayerHand>,
    dealer_hand: &DealerHand,
    dispenser: &mut CardDispenser,
    player_chips: &mut u32,
    surrender: &Surrender,
) {
    while let Some(hand) = hands.iter_mut().find(|hand| !hand.is_over()) {
        pause();
        println!(
            "What would you like to do? ({} against {})",
            hand,
            dealer_hand.showing()
        );
        match make_move(
            hand.len(),
            hand.is_pair(),
            *player_chips >= hand.bet(),
            surrender,
        ) {
            Action::Stand => {
                println!("You stand!");
                hand.stand();
            }
            Action::Hit => {
                println!("You hit!");
                *hand += dispenser.draw_card();
            }
            Action::DoubleDown => {
                println!("You double and put another {} chips down!", hand.bet());
                *player_chips -= hand.bet(); // The player pays another equal bet
                hand.double(dispenser.draw_card());
            }
            Action::Split => {
                println!("You split your hand and put another {} chips down!", hand.bet());
                *player_chips -= hand.bet(); // The player pays another equal bet for the new hand
                let mut new_hand = PlayerHand::split(hand);
                *hand += dispenser.draw_card();
                new_hand += dispenser.draw_card();
                hands.push(new_hand);
            }
            Action::Surrender => {
                println!("You surrender!");
                hand.surrender();
            }
        }
    }
}

fn pause() {
    thread::sleep(Duration::from_secs(1));
}
