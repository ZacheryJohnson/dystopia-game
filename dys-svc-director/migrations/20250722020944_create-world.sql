CREATE TABLE corporation (
    corp_id INT UNSIGNED NOT NULL PRIMARY KEY AUTO_INCREMENT,
    name TEXT NOT NULL
) AUTO_INCREMENT = 1;

CREATE TABLE combatant (
    combatant_id INT UNSIGNED NOT NULL PRIMARY KEY AUTO_INCREMENT,
    corp_id INT UNSIGNED,
    name TEXT NOT NULL,
    serialized_combatant MEDIUMBLOB NOT NULL,

    FOREIGN KEY (corp_id) REFERENCES corporation(corp_id)
) AUTO_INCREMENT = 1;

CREATE TABLE season (
    season_id INT UNSIGNED NOT NULL PRIMARY KEY AUTO_INCREMENT
) AUTO_INCREMENT = 1;

CREATE TABLE game (
    game_id INT UNSIGNED NOT NULL PRIMARY KEY AUTO_INCREMENT,
    season_id INT UNSIGNED,
    team_1 INT UNSIGNED NOT NULL,
    team_2 INT UNSIGNED NOT NULL,

    FOREIGN KEY (season_id) REFERENCES season(season_id),
    FOREIGN KEY (team_1) REFERENCES corporation(corp_id),
    FOREIGN KEY (team_2) REFERENCES corporation(corp_id)
) AUTO_INCREMENT = 1;

CREATE TABLE game_schedule (
    game_id INT UNSIGNED NOT NULL,
    start_time_utc DATETIME NOT NULL,

    FOREIGN KEY (game_id) REFERENCES game(game_id)
);

CREATE TABLE game_results (
    game_id INT UNSIGNED NOT NULL PRIMARY KEY,
    serialized_results MEDIUMBLOB NOT NULL,

    FOREIGN KEY (game_id) REFERENCES game(game_id)
);