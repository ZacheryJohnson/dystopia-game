<script setup lang="ts">
import { getMatchVisualizerStore } from "@/stores/MatchVisualizer";
import { computed } from "vue";
const matchVisualizerStore = getMatchVisualizerStore();

const props = defineProps([
    "awayAbbr",
    "homeAbbr",
    "awayScore",
    "homeScore",
    "gameLogData",
])

const isSelected = computed(() => matchVisualizerStore.gameLogData == props.gameLogData);

function onElementClicked() {
    matchVisualizerStore.gameLogData = props.gameLogData;
}

const gameOver = true; // ZJ-TODO: calculate this
const awayWin = gameOver && props.awayScore > props.homeScore;
const homeWin = gameOver && props.homeScore > props.awayScore;

const getTeamNameFn = (abbr: string) => {
  switch (abbr) {
    case "ALP": return "alpha";
    case "BET": return "beta";
    case "DEL": return "delta";
    case "GAM": return "gamma";
  }
}

const awayTeamImgPath = `/images/teams/team_wip_${getTeamNameFn(props.awayAbbr)}.png`;
const homeTeamImgPath = `/images/teams/team_wip_${getTeamNameFn(props.homeAbbr)}.png`;
</script>

<template>
  <div class="game" :class="{'selected': isSelected}" @click="onElementClicked()">
    <img :src="awayTeamImgPath" alt="Away Team Logo"/>
    <p class="teamName" :class="{ 'winner-text': awayWin }">{{ awayAbbr }}</p>
    <p class="record" :class="{ 'winner-text': awayWin }">(0-0)</p>
    <p :class="{ 'winner-text': awayWin }">{{ awayScore }}</p>

    <img :src="homeTeamImgPath" alt="Home Team Logo"/>
    <p class="teamName" :class="{ 'winner-text': homeWin }">{{ homeAbbr }}</p>
    <p class="record" :class="{ 'winner-text': homeWin }">(0-0)</p>
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

@media (prefers-color-scheme: dark) {
  .selected {
    border-color: darkgray;
    background-color: dimgray;
  }
}

.game {
  border-width: 1px;
  border-style: solid;
  border-radius: 10px;

  display: grid;
  grid-template-rows: 50% 50%;
  grid-template-columns: 20% 25% 27.5% 27.5%;
  column-gap: 3%;

  align-items: center;
  padding: 2px 15px;
  margin: 10px 5px;
}

.game img {
  display: block;
  margin-left: auto;
  max-width: 50%;
  max-height: 50%;
  width: auto;
  height: auto;
}

.teamName {
  text-align: center;
}

.record {
  text-align: left;
  font-size: 75%;
}

.winner-text {
  font-weight: bold;
}
</style>