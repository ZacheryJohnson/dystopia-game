<script setup lang="ts">
import { getMatchVisualizerStore } from '@/stores/MatchVisualizer'
import {computed, onMounted, ref} from 'vue'
import type {GetGameLogResponse} from "%/services/match_results/summary.ts";
import {getSeasonStore} from "@/stores/Season.ts";
const matchVisualizerStore = getMatchVisualizerStore()
const seasonStore = getSeasonStore();

const props = defineProps([
    'matchId',
    'awayAbbr',
    'homeAbbr',
    'awayScore',
    'homeScore',
    'awayRecord',
    'homeRecord',
    'dateStr',
])

const isSelected = computed(() => matchVisualizerStore.selectedMatchId == props.matchId);

const timeFormat = new Intl.DateTimeFormat(
  undefined, // undefined = runtime default
  {
    "hour": "numeric",
    "minute": "2-digit",
  }
);

async function onElementClicked() {
  const response: GetGameLogResponse = await (
      await fetch(`api/game_log/${props.matchId}`)
  ).json();

  matchVisualizerStore.gameLogData = response.gameLogSerialized!;
  matchVisualizerStore.selectedMatchId = props.matchId;
}

onMounted(async() => {
  await seasonStore.fetchSeason();
});

const gameOver = true // ZJ-TODO: calculate this
const awayWin = gameOver && props.awayScore > props.homeScore
const homeWin = gameOver && props.homeScore > props.awayScore

const getScheduledTimeFn = () => {
  const matches = getSeasonStore().season.get(props.dateStr);
  if (!matches) {
    return "";
  }

  for (const match of matches) {
    if (match.matchId === props.matchId) {
      const utcSeconds = match.utcScheduledTime || 0;
      const time = new Date(utcSeconds * 1000);
      return utcSeconds ? timeFormat.format(time) : "";
    }
  }

  return "";
}

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
      <div class="game-schedule-time" :class="!awayScore && !awayScore ? 'upcoming-match' : ''">
        <span>{{!awayScore && !homeScore && getScheduledTimeFn().length > 0 ? getScheduledTimeFn() : ""}}</span>
      </div>

      <img :src="awayTeamImgPath" alt="Away Team Logo" />
      <p class="teamName" :class="{ 'winner-text': awayWin }">{{ awayAbbr }}</p>
      <template v-if="awayScore">
        <p class="record" :class="{ 'winner-text': awayWin }">({{ props.awayRecord }})</p>
        <p :class="{ 'winner-text': awayWin }">{{ awayScore ? awayScore : "" }}</p>
      </template>
      <template v-else>
        <p
          class="record"
          :class="{ 'winner-text': awayWin }"
          style="grid-area: auto / span 2;"
        >
          ({{ props.awayRecord }})
        </p>
      </template>

      <img :src="homeTeamImgPath" alt="Home Team Logo" />
      <p class="teamName" :class="{ 'winner-text': homeWin }">{{ homeAbbr }}</p>

      <template v-if="homeScore">
        <p class="record" :class="{ 'winner-text': homeWin }">({{ props.homeRecord }})</p>
        <p :class="{ 'winner-text': homeWin }" :style="homeScore ? '' : 'span 0'">{{ homeScore ? homeScore : "" }}</p>
      </template>
      <template v-else>
        <p
            class="record"
            :class="{ 'winner-text': homeWin }"
            style="grid-area: auto / span 2;"
        >
          ({{ props.homeRecord }})
        </p>
      </template>
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

.upcoming-match {
  background-color: #00bd7e;
  border-width: 1px;
  border-radius: 10px;
  text-align: center;
}

.game-schedule-time {
  grid-area: auto / span 2 / auto / 5;
}

.game {
    border-width: 1px;
    border-style: solid;
    border-radius: 10px;

    display: grid;
    grid-template-rows: 14% 43% 43%;
    grid-template-columns: 9% 31% 36.5% 23.5%;
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