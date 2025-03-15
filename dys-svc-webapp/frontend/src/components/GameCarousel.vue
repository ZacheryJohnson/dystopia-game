<script setup lang="ts">
import {ref, onMounted, type Ref, computed} from "vue";
  import GameCarouselElement from "./GameCarouselElement.vue";
import {
  type GetGameLogResponse,
  MatchResponse_MatchSummary as MatchSummary
} from "%/services/match_results/summary.ts";
import {date_MonthToJSON, DateMessage} from "%/common/date.ts";
import {getSeasonStore} from "@/stores/Season.ts";

  const seasonStore = getSeasonStore();
  const gameLogs: Ref<Map<number, Uint8Array>> = ref(new Map());
  const hasMatches = computed(() => seasonStore.matchesByDate.size > 0);

  const dateToStr = (date: DateMessage) => {
    return `${date.year}-${date.month.valueOf()}-${date.day}`;
  };

  const dateFromStr = (str: string): DateMessage => {
    const date = DateMessage.create();

    const parts = str.split('-');
    const yearStr = parts[0];
    const monthStr = parts[1];
    const dayStr = parts[2];

    date.year = parseInt(yearStr);
    date.month = parseInt(monthStr);
    date.day = parseInt(dayStr);

    return date;
  };

  onMounted(async () => {
    const matchSummaries: MatchSummary[] = (await (await fetch(`api/summaries`)).json()).matchSummaries;

    seasonStore.matchesByDate = new Map();
    gameLogs.value = new Map();
    for (const match of matchSummaries) {
      const response: GetGameLogResponse = (await (await fetch(`api/game_log/${match.matchId}`)).json());
      gameLogs.value.set(match.matchId, response.gameLogSerialized);

      const dateStr = dateToStr(match.date!);
      if (seasonStore.matchesByDate.has(dateStr)) {
        seasonStore.matchesByDate.get(dateStr)!.push(match);
      } else {
        seasonStore.matchesByDate.set(dateStr, [match]);
      }
    }
  });
</script>

<template>
  <div class="carousel-frame">
    <template v-if="hasMatches" v-for="[dateStr, matches] of seasonStore.matchesByDate">
      <div class="date-block" :id="dateStr">
        <span class="date-year">{{dateFromStr(dateStr).year}}</span>
        <br>
        <span class="date-month">{{date_MonthToJSON(dateFromStr(dateStr).month)}}</span>
        <br>
        <span class="date-day">{{dateFromStr(dateStr).day}}</span>
      </div>
      <GameCarouselElement
          v-for="match of matches"
          :key="match.matchId"
          :awayAbbr="match.awayTeamName.substring(0, 3).toUpperCase()"
          :homeAbbr="match.homeTeamName.substring(0, 3).toUpperCase()"
          :awayScore="match.awayTeamScore"
          :homeScore="match.homeTeamScore"
          :awayRecord="match.awayTeamRecord"
          :homeRecord="match.homeTeamRecord"
          :gameLogData="gameLogs.get(match.matchId)"
      />
    </template>
    <template v-else>
      <div></div>
      <p>No matches! Check back soon.</p>
    </template>
  </div>
</template>

<style scoped>
.carousel-frame {
  display: grid;
  grid-auto-flow: column;
  grid-auto-columns: minmax(60px, .0264fr);
  grid-gap: 5px;
  min-height: 100px;

  border-bottom: 2px solid;
  overflow: scroll;
}

.carousel-frame .game {
  grid-column: span 3;
}

.date-block {
  height: auto;
  width: max-content;
  border-style: solid;
  border-width: 1px;
  text-align: center;
}

.date-year {
  font-size: 16px;
}

.date-month {
  font-size: 12px;
}

.date-day {
  font-size: 24px;
}
</style>