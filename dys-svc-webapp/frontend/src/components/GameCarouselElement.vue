<script setup lang="ts">
import { getMatchVisualizerStore } from '@/stores/MatchVisualizer'
import { computed } from 'vue'
import type {GetGameLogResponse} from "%/services/match_results/summary.ts";
const matchVisualizerStore = getMatchVisualizerStore()

const props = defineProps([
    'matchId',
    'awayAbbr',
    'homeAbbr',
    'awayScore',
    'homeScore',
    'awayRecord',
    'homeRecord'
])

const isSelected = computed(() => matchVisualizerStore.selectedMatchId == props.matchId)

async function onElementClicked() {
  const response: GetGameLogResponse = await (
      await fetch(`api/game_log/${props.matchId}`)
  ).json();

  matchVisualizerStore.gameLogData = response.gameLogSerialized;
  matchVisualizerStore.selectedMatchId = props.matchId;
}

const gameOver = true // ZJ-TODO: calculate this
const awayWin = gameOver && props.awayScore > props.homeScore
const homeWin = gameOver && props.homeScore > props.awayScore

const getTeamNameFn = (abbr: string) => {
    switch (abbr) {
        case 'ALP':
            return 'alpha'
        case 'BET':
            return 'beta'
        case 'DEL':
            return 'delta'
        case 'GAM':
            return 'gamma'
    }
}

const awayTeamImgPath = `/images/teams/team_wip_${getTeamNameFn(props.awayAbbr)}.png`
const homeTeamImgPath = `/images/teams/team_wip_${getTeamNameFn(props.homeAbbr)}.png`
</script>

<template>
    <div class="game" :class="{ selected: isSelected }" @click="async () => await onElementClicked()">
        <img :src="awayTeamImgPath" alt="Away Team Logo" />
        <p class="teamName" :class="{ 'winner-text': awayWin }">{{ awayAbbr }}</p>
        <p class="record" :class="{ 'winner-text': awayWin }">({{ props.awayRecord }})</p>
        <p :class="{ 'winner-text': awayWin }">{{ awayScore }}</p>

        <img :src="homeTeamImgPath" alt="Home Team Logo" />
        <p class="teamName" :class="{ 'winner-text': homeWin }">{{ homeAbbr }}</p>
        <p class="record" :class="{ 'winner-text': homeWin }">({{ props.homeRecord }})</p>
        <p :class="{ 'winner-text': homeWin }">{{ homeScore }}</p>
    </div>
</template>

<style scoped>
p {
    font-family: 'VarelaRound';
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
    grid-template-columns: 11% 28% 30.5% 30.5%;
    column-gap: 3%;

    align-items: center;
    padding: 2px 15px;
    margin: 10px 5px;
}

.game img {
    display: block;
    margin-left: auto;
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