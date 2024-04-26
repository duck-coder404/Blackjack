# Blackjack

This is a Blackjack game/simulator that I made in Rust, implemented via a state machine.
My goal with this project is to enable users to investigate the difference in the house edge between arbitrary table rules, by running millions of concurrent simulations.
One should be able to start two concurrent simulations with slightly different rules and see how the results change in the long run.

The project consists of three separate crates:

- `blackjack-core`: The backend functionality, including the state machine and data object model.
- `blackjack-cli`: A simple command-line frontend. This was the original format of the application.
- `blackjack-gui`: A more advanced GUI application built using [Ratatui](https://github.com/ratatui-org/ratatui).

## Features

- [x] Fully-featured Blackjack gameplay
- [x] Highly configurable
- [x] Surrendering (early and late)
- [x] Insurance (even though it's a bad idea)
- [x] Simulation with Basic Strategy
- [x] (GUI) Many simultaneous games
- [x] (GUI) Continuous game statistics

## TODOs

- [ ] Display more visuals in the GUI
- [ ] Switch between gameplay and simulation on-the-fly
- [ ] Rule-adaptive basic strategy
