# CSE 485 Poker Solver

This repository will be used for the CSE485 capstone project.

## Project Description

Our goal for this project is to create an application that will use reinforcement learning and self-play to create an un-exploitable or unbeatable strategy for Heads up no limit Texas Holdem'.  The program will learn to play poker by updating its strategy using an algorithm known as counterfactual regret

## Directories

- **notebooks** - contains IPython notebook tutorials and explanations for various components of the project
- **poker_solver** - contains a cargo (Rust) project with the code for our solver

## Sprints

### Sprint 1: Console Game Environment

For sprint one our goal is to create an environent to interface with our agent.  We will create a console application to simulate a game of HUNL Poker.  To do this we'll need to build a few things.

 - **Game State** will be rust structure to represent that game state.  Will have variables such as *pot_size*, *current_player*, *player_has_folded*, ect.
 - We will likely need a **Player State** object to be stored within the State.  This class will hold information such as *player_cards*, *stack*, and *wager*
 - The Game State object will required a number of function to aid in state transitions.
   * **get_valid_actions**will return an array of valid actions that the current player can take.
   * **apply_action** will apply an action and update the state of the game.
 - **Environment** we will to create a console environment to interface with the user who will supply actions to the game state.  This will show them a menu of available actions to take and information such as their cards and stack size.
