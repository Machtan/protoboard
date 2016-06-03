# Initiative

Every turn a player can move up to 4 units.

Units cannot perform a ranged attack the same turn they moved.

# Units

## Stats

The following four stats are planned:

* *Attack power (AP)*: Scales the damage dealt when attacking (and
  defending?)
* *Movement (MV)*: The number of tile points a unit can move. Terrain
  can change this, and certain units might have movement bonuses on some
  terrain types.
* *Hit points (HP)*: How much damage a unit can soak before being out of
  commission.
* *Range (RN)*: How many tiles away the unit can target and attack
  other units.

Different units' stats could be allocated thusly:

    Unit type | AP  | MV  | HP  | RN
    ----------+-----+-----+-----+-----
    Defender  |  1  |  3  |  8  |  1
    Warrior   |  2  |  3  |  5  |  1
    Archer    |  2  | 4/5 |  5  | 2-3
    Spearman  |  2  |  3  |  4  | 1-2*

\* For these units, ranged range only applies, if there are no units
  blocking line-of-sight.
