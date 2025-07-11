# Dystopia AI

## Overview

Dystopia uses GOAP (Goal Oriented Action Planning) for it's AI systems.
In the abstract, AI agents choose goals to achieve, and achieve those 
goals by constructing a series of contiguous actions called a plan.

### Agents

In Dystopia, our agents are combatants. Each combatant implements
the [Agent](agent.rs) trait, but actions, goals, strategies, sensors, 
etc do not operate on combatants, but rather Agent implementations.
This is to enable non-combatant agents in the future, but none are 
currently planned.

### Goals

### Plans

### Actions

### Strategies

### Beliefs

## Teamplay

### Abstract

One area where GOAP is typically viewed as less than ideal is in
"team coordination" settings, as agents typically plan for themselves
using their own set of beliefs. This would make GOAP a suboptimal
choice for projects that are team-vs-team sports, but here we are - 
blame past Zach.

Coordination between teammates in real sports is largely predicated
on pre-practiced plays (or set pieces if you're a footy fan).
In something like basketball, a point guard brings the ball up the 
court, and the players know what actions they should perform for the
remainder of the possession, or until a shot is taken and the ball is
reset. Baseball by comparison is far more improvisational, where the
furthest a set play gets is a suicide squeeze or hit-and-run, but 
failing to execute those plays as practiced will result in individual
improvisation to where communication between players is far less
necessary.

To achieve teamplay in a GOAP setting, we need to establish the concept
of a "shared belief", where each agent believes that other agents
involved in the play are aware of their own participation. In basketball
terms, the point guard assumes that the center will set a pick on the
point guard's defender's left, and the point guard will then go around
the screen - if either player fails their end of the play's responsibilities,
the play fails.

Beyond set plays, sports team also have formations from which plays are 
executed from. One common soccer formation is the 4-4-2, which is 4 defense,
4 midfield, and 2 forwards. This dictates a player's expected role over the
course of a game, where improvisational moments allow the player to leave that
role temporarily with the expectation that they'll return to that role sooner
than later.

### Dystopia Implementation

From these sports analogies, we derive two key implementation details:
- all combatants of a team will have a non-expiring TeamFormation belief
- combatants may call for and execute plays

#### Formations

Formations are beliefs that include a combatant's role within the team. This
belief influences where a combat will tend to stand in the arena and what other
actions will be prioritized.

#### Plays

Plays are called by a combatant via actions, and received by one or more
combatants via sensors. 