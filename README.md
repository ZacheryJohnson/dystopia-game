# Dystopia (working title)

Dystopia is a fantasy sports simulation in which megacorporations sponsor teams that compete in violent sport.

Each member of the team, called a combatant, is a clone, the genetic blueprints of which are owned by the corporations.

## The Sport (no title)

The sport is a violent combination of dodgeball, king of the hill, and paintball. Each arena has uniquely placed obstacles that combatants can hide behind (paintball) and designated areas where combatants must stand to achieve points for their team (king of the hill). 

Teams achieve points by having combatants on designated scoring zones ("plates"). Every second, all plates reward 1 point to each combatant on the plate. If no opponents are on the plate, it awards twice the points. However, any combatant holding a ball (see next section) will not be scored: this is to disincentivize being offensive on plates.

The game is comprised of two halves, the first of which goes until 75 points or 5 minutes (whichever occurs first) and the second until 150 points or 5 minutes (whichever occurs first).

Throughout the arena are balls that can be passed to teammates or thrown at opponents. When a ball is picked up by a player, it charges up by a flat amount. When a ball is not being touched by a player, it loses charge over time. When a thrown ball hits an opponent, it explodes based on it's charge, applying a concussive force and knocks players back based on the explosion point (affecting both teammates and opponents). Balls will also lose charge rapidly after 3 seconds of being held by the same player, and will self-detonate after being held for 7 seconds: this is to disincentivize simply holding a ball and to encourage throwing at opponents or passing to teammates. Unlike dodgeball, catching a ball does not mitigate the "damage", and the ball will still detonate.

This creates a few strategic opportunities:
- passing balls between teammates will charge them up more (accurate throws + teamplay matter)
- stronger thrown balls will lose less charge while in the air than weaker thrown balls (throw strength matters)
- dodging throws is important, as only direct hits will detonate (dexterity and smart positioning matter)

The ball explosion is very dangerous, and can injure, main, debilitate, and otherwise harm combatants. Combatants may not be replaced during a half, but may be swapped at halftime. This means combatants that are injured (potentially fatally so) must remain in the arena and contribute however they can with their injuries (unless dead, in which they just take up space).

### Statistics

There are many statistics that are tracked during a game, which can indicate combatants' strengths and weaknesses.

#### Individual direct stats

- Points scored (**PS**): points earned by standing on a plate
- Balls picked up (**BU**): balls picked up off the ground
- Balls passed (**BP**): balls passed to teammates
- Balls thrown (**BT**): balls thrown at opponents
- Balls reset (**BR**): balls picked up by a combatant that had opposing charge when picked up, resetting the ball to 0 charge
- Balls whiffed (**BW**): balls thrown at opponents that did not hit the opponent
- Hits direct (**HD**): balls thrown that hit an opponent directly
- Hits indirect (**HI**): balls thrown that hit an opponent indirectly (through explosion radius)
- Hits assisted (**HA**): any hit in which the combatant did not throw the ball but provided charge to the hit
- Plates displaced (**PD**): any action that results in an opponent being removed from a plate, one per opponent removed (an explosion can remove multiple opponents, resulting in multiple PDs)
- Plates cleared (**PC**): any action that results in ALL opponents being removed from a plate
- Halves completed (**H**): awarded when the combatant survives a half (injuries okay)

#### Individual calculated stats

- Hit rate (**HR**): `HD / BT` OR `(BT - BW) / BT`
- Throw efficiency (**TE**): `(HD + HI) / BT`
- Quality throw score (**QTS**): `PD / BT`
- Points per game (**PPG**): `PS / (2 * H)`

#### Team stats
- Points for (**PF**): sum of team's combatants' **PS**
- Points allowed (**PA**): sum of opponent's combatants' **PS**
- Halves score limit reached for (**HSLF**): number of halves where the score limit was reached by this team
- Halves score limit reached for (**HSLA**): number of halves where the score limit was reached by the opponent team
- Halves time limit reached (**HTL**): number of halves where the time limit was reached