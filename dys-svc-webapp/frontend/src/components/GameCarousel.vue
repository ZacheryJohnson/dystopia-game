<script setup lang="ts">
import {ref, onMounted, type Ref, computed} from "vue";
  import GameCarouselElement from "./GameCarouselElement.vue";
import {
  GameSummaryResponse_GameSummary as GameSummary
} from "%/services/game_results/summary.ts";
import {date_MonthToJSON, DateMessage} from "%/common/date.ts";
import {getSeasonStore} from "@/stores/Season.ts";

  const seasonStore = getSeasonStore();
  const hasGames = computed(() => seasonStore.gamesByDate.size > 0);

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
    const payload = await (await fetch(`api/summaries`)).json();
    const gameSummaries: GameSummary[] = payload.gameSummaries;
    const nextGames: GameSummary[] = payload.nextGames;

    seasonStore.gamesByDate = new Map();

    for (const game of nextGames) {
      const dateStr = dateToStr(game.date!);
      if (seasonStore.gamesByDate.has(dateStr)) {
        seasonStore.gamesByDate.get(dateStr)!.push(game);
      } else {
        seasonStore.gamesByDate.set(dateStr, [game]);
      }
    }

    for (const game of gameSummaries) {
      const dateStr = dateToStr(game.date!);
      if (seasonStore.gamesByDate.has(dateStr)) {
        seasonStore.gamesByDate.get(dateStr)!.push(game);
      } else {
        seasonStore.gamesByDate.set(dateStr, [game]);
      }
    }
  });
</script>

<template>
  <div class="carousel-frame">
    <template v-if="hasGames" v-for="[dateStr, games] of seasonStore.gamesByDate">
      <div class="date-block" :id="dateStr">
        <span class="date-year">{{dateFromStr(dateStr).year}}</span>
        <br>
        <span class="date-month">{{date_MonthToJSON(dateFromStr(dateStr).month)}}</span>
        <br>
        <span class="date-day">{{dateFromStr(dateStr).day}}</span>
      </div>
      <GameCarouselElement
          v-for="game of games"
          :key="game.gameId"
          :gameId="game.gameId"
          :awayAbbr="game.awayTeamName?.substring(0, 3).toUpperCase()"
          :homeAbbr="game.homeTeamName?.substring(0, 3).toUpperCase()"
          :awayScore="game.awayTeamScore"
          :homeScore="game.homeTeamScore"
          :awayRecord="game.awayTeamRecord"
          :homeRecord="game.homeTeamRecord"
          :dateStr="dateStr"
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