<script setup lang="ts">
const props = defineProps([
    "away_abbr",
    "home_abbr",
    "away_score",
    "home_score",
])

const game_over = true; // ZJ-TODO: calculate this
const away_win = game_over && props.away_score > props.home_score;
const home_win = game_over && props.home_score > props.away_score;
</script>

<template>
    <div class="game">
        <!-- TODO: team logo -->
        <p todo="replace with team logo"></p>
        <p :class="{ 'winner-text': away_win }">{{ away_abbr }}</p>
        <p :class="{ 'winner-text': away_win }">{{ away_score }}</p>

        <!-- TODO: team logo -->
        <p todo="replace with team logo"></p>
        <p :class="{ 'winner-text': home_win }">{{ home_abbr }}</p>
        <p :class="{ 'winner-text': home_win }">{{ home_score }}</p>
    </div>
</template>

<style>
p {
    font-family: "VarelaRound";
}

.game {
    border-width: 2px;
    border-style: solid;
    border-radius: 10px;

    text-align: right;
    display: grid;
    grid-row: auto auto;
    grid-template-rows: 50% 50%;
    grid-template-columns: 15% 50% 35%;

    padding: 2px 15px;
    margin: 5px 5px;
}

.winner-text {
    font-weight: bold;
}
</style>