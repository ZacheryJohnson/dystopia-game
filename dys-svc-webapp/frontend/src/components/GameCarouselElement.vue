<script setup lang="ts">
import { getMatchVisualizerStore } from "@/stores/MatchVisualizer";
import { computed } from "vue";
const matchVisualizerStore = getMatchVisualizerStore();

const props = defineProps([
    "awayAbbr",
    "homeAbbr",
    "awayScore",
    "homeScore",
    "gameLogPath",
])

const isSelected = computed(() => matchVisualizerStore.gameLogPath == props.gameLogPath);

function onElementClicked() {
    matchVisualizerStore.gameLogPath = props.gameLogPath;
}

const gameOver = true; // ZJ-TODO: calculate this
const awayWin = gameOver && props.awayScore > props.homeScore;
const homeWin = gameOver && props.homeScore > props.awayScore;
</script>

<template>
    <div class="game" :class="{'selected': isSelected}" @click="onElementClicked()">
        <!-- TODO: team logo -->
        <p todo="replace with team logo"></p>
        <p :class="{ 'winner-text': awayWin }">{{ awayAbbr }}</p>
        <p :class="{ 'winner-text': awayWin }">{{ awayScore }}</p>

        <!-- TODO: team logo -->
        <p todo="replace with team logo"></p>
        <p :class="{ 'winner-text': homeWin }">{{ homeAbbr }}</p>
        <p :class="{ 'winner-text': homeWin }">{{ homeScore }}</p>
    </div>
</template>

<style>
p {
    font-family: "VarelaRound";
}

.selected {
    border-color: green;
    background-color: lightcyan;
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