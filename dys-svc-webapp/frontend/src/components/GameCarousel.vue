<script setup lang="ts">
import {ref, onMounted, type Ref, computed} from "vue";
  import GameCarouselElement from "./GameCarouselElement.vue";
import {
  MatchResponse_MatchSummary as MatchSummary
} from "%/services/match_results/summary.ts";
import {date_MonthToJSON, DateMessage} from "%/common/date.ts";

  type DateAndMatchesT = Map<string, MatchSummary[]>;
  const dateAndMatches: Ref<DateAndMatchesT> = ref(new Map());
  const hasMatches = computed(() => dateAndMatches.value.size > 0);

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

    dateAndMatches.value = new Map();
    for (const match of matchSummaries) {
      const dateStr = dateToStr(match.date!);
      if (dateAndMatches.value.has(dateStr)) {
        dateAndMatches.value.get(dateStr)!.push(match);
      } else {
        dateAndMatches.value.set(dateStr, [match]);
      }
    }
  });
</script>

<template>
  <div class="carousel-frame" v-if="hasMatches" v-for="[dateStr, matches] of dateAndMatches">
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
        :gameLogData="match.gameLogSerialized"
    />
  </div>
  <div class="carousel-frame" v-else>
    <p>No matches! Check back soon.</p>
  </div>
</template>

<style scoped>
.carousel-frame {
  display: grid;
  grid-auto-flow: column;
  grid-auto-columns: minmax(75px, .033fr);
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
  padding: 5px;
  margin: 0 5px;
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