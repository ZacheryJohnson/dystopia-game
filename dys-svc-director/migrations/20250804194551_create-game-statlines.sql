CREATE TABLE game_statline (
    game_id INT UNSIGNED NOT NULL,
    combatant_id INT UNSIGNED NOT NULL,
    points INT,
    balls_thrown INT UNSIGNED,
    throws_hit INT UNSIGNED,
    combatants_shoved INT UNSIGNED,

    PRIMARY KEY (game_id, combatant_id),
    FOREIGN KEY (game_id) REFERENCES game(game_id),
    FOREIGN KEY (combatant_id) REFERENCES combatant(combatant_id)
);