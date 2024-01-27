use crate::card::dispenser::Shoe;
use crate::card::hand::{PlayerHand, DealerHand, HandStatus};
use crate::Configuration;
use crate::statistics::Statistics;
use crate::input::{Player, GameAction, HandAction};

pub struct Game {
    pub dispenser: Shoe,
    pub soft_17_hit: bool,
    pub six_to_five: bool,
    pub min_bet: Option<u32>,
    pub max_bet: Option<u32>,
    pub early_surrender: bool,
    pub late_surrender: bool,
    pub split_aces: bool,
    pub double_after_split: bool,
    pub max_splits: Option<u8>,
    pub insurance: bool,
    pub turns: Vec<EndTurn>,
}

pub struct StartTurn {
    pub player_hand: PlayerHand,
    pub dealer_hand: DealerHand,
    pub insurance: u32,
}

pub struct EndTurn {
    pub player_hands: Vec<PlayerHand>,
    pub dealer_hand: DealerHand,
    pub insurance: u32,
    pub total_bet: u32,
    pub winnings: u32,
}

impl Game {
    pub fn new(config: Configuration) -> Self {
        Self {
            dispenser: Shoe::new(config.decks, config.penetration),
            soft_17_hit: config.soft_17_hit,
            six_to_five: config.six_to_five,
            min_bet: config.min_bet,
            max_bet: config.max_bet,
            early_surrender: config.early_surrender,
            late_surrender: config.late_surrender,
            split_aces: config.split_aces,
            double_after_split: config.double_after_split,
            max_splits: config.max_splits,
            insurance: config.insurance,
            turns: Vec::new(),
        }
    }

    pub fn play(mut self, player: &mut Player) {
        println!("Welcome to Blackjack!");
        let mut stats = Statistics::new();
        while let GameAction::Bet(bet) = player.place_bet_or_quit(&self) {
            println!("You bet {} chips. You have {} chips remaining.", bet, player.chips);
            player.wait();
            let turn = self.start_turn(player, bet);
            let mut turn = self.play_hands(player, turn);
            self.payout(player, &mut turn);
            if player.chips < self.min_bet.unwrap_or(1) {
                println!("You don't have enough chips to continue!");
                break;
            }
            self.shuffle_cards_if_needed(player);
            stats.update(&turn);
            self.turns.push(turn);
        }
        println!("You finished with {} chips.", player.chips);
        println!("Goodbye!");
        println!("Game statistics: {}", stats);
        player.wait();
    }

    fn start_turn(&mut self, player: &mut Player, bet: u32) -> StartTurn {
        let mut player_hand = PlayerHand::new(self.dispenser.draw_card(), bet);
        player.wait();
        let mut dealer_hand = DealerHand::new(self.dispenser.draw_card(), self.soft_17_hit);
        player.wait();

        player_hand += self.dispenser.draw_card();
        player.wait();
        dealer_hand += self.dispenser.draw_card();
        player.wait();

        let mut insurance = 0;
        if dealer_hand.showing() >= 10 {
            if self.early_surrender && player.surrender_early(self, &player_hand, &dealer_hand) {
                println!("You surrender!");
                player_hand.surrender();
                player.wait();
            } else if self.insurance && dealer_hand.showing() == 11 {
                insurance = player.offer_insurance(player_hand.bet / 2);
                player.chips -= insurance;
                player.wait();
            }
            println!("The dealer checks their hand for blackjack...");
            player.wait();
        }

        StartTurn {
            player_hand,
            dealer_hand,
            insurance
        }
    }

    fn play_hands(&mut self, player: &mut Player, mut turn: StartTurn) -> EndTurn {
        // The player may now play their hand, which may turn into multiple hands through splitting
        // (skip if dealer has blackjack)
        let mut player_hands = if turn.dealer_hand.status == HandStatus::Blackjack {
            vec![turn.player_hand]
        } else {
            self.play_player_hand(player, turn.player_hand, &turn.dealer_hand)
        };
        // The dealer reveals their hole card
        turn.dealer_hand.reveal_hole_card();
        player.wait();
        // The dealer plays their hand
        if player_hands.iter().any(|hand| hand.status == HandStatus::Stood) {
            // At least one hand was played and stood on, so the dealer must finish their hand
            self.play_dealer_hand(player, &mut turn.dealer_hand)
        }
        player_hands.iter_mut().for_each(|hand| hand.calculate_winnings(&turn.dealer_hand, self.six_to_five));
        let total_bet = player_hands.iter().map(|hand| hand.bet).sum();
        let winnings = player_hands.iter().map(|hand| hand.winnings).sum();
        EndTurn {
            player_hands,
            dealer_hand: turn.dealer_hand,
            insurance: turn.insurance,
            total_bet,
            winnings,
        }
    }

    fn play_player_hand(&mut self, player: &mut Player, player_hand: PlayerHand, dealer_hand: &DealerHand) -> Vec<PlayerHand> {
        let mut player_hands = vec![player_hand];
        while let Some(player_hand) = player_hands.iter_mut().find(|hand| hand.status == HandStatus::InPlay) {
            println!(
                "{} against {}.",
                player_hand.value,
                dealer_hand.showing()
            );
            match player.get_hand_action(
                self,
                player_hand,
                dealer_hand,
            ) {
                HandAction::Stand => {
                    println!("You stand!");
                    player_hand.stand();
                }
                HandAction::Hit => {
                    println!("You hit!");
                    player.wait();
                    *player_hand += self.dispenser.draw_card();
                }
                HandAction::Double => {
                    println!("You double and put another {} chips down!", player_hand.bet);
                    player.wait();
                    player.chips -= player_hand.bet; // The player pays another equal bet
                    player_hand.double(self.dispenser.draw_card());
                }
                HandAction::Split => {
                    println!(
                        "You split your hand and put another {} chips down!",
                        player_hand.bet
                    );
                    player.chips -= player_hand.bet; // The player pays another equal bet for the new hand
                    let mut new_hand = player_hand.split();
                    player.wait();
                    *player_hand += self.dispenser.draw_card();
                    player.wait();
                    new_hand += self.dispenser.draw_card();
                    player_hands.push(new_hand);
                }
                HandAction::Surrender => {
                    println!("You surrender!");
                    player_hand.surrender();
                }
            }
            player.wait();
        }
        player_hands
    }

    fn play_dealer_hand(&mut self, player: &Player, dealer_hand: &mut DealerHand) {
        // At least one hand was played and stood on, so the dealer must finish their hand
        while dealer_hand.status == HandStatus::InPlay {
            *dealer_hand += self.dispenser.draw_card();
            player.wait();
        }
    }

    fn payout(&mut self, player: &mut Player, turn: &mut EndTurn) {
        if turn.insurance > 0 && turn.dealer_hand.status == HandStatus::Blackjack {
            turn.winnings += turn.insurance * 2;
        }

        match turn.winnings {
            0 => println!("You lose!"),
            chips if chips < turn.total_bet => println!("You make back {} chips!", chips),
            chips if chips == turn.total_bet => println!("You push!"),
            chips => println!("You win {chips} chips!"),
        }

        player.chips += turn.winnings;
        player.wait();
    }

    fn shuffle_cards_if_needed(&mut self, player: &Player) {
        if self.dispenser.needs_shuffle() {
            println!("The dealer shuffles the cards.");
            self.dispenser.shuffle();
            player.wait();
        }
    }

}