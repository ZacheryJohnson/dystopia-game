<script setup lang="ts">

import {computed, onMounted, type Ref, ref} from "vue";
import {DateMessage} from "%/common/date.ts";
import {getSeasonStore} from "@/stores/Season.ts";

const getDateFromDateStr = (dateStr: string): DateMessage => {
  const parts = dateStr.split("-");
  return DateMessage.fromJSON({
    year: parseInt(parts[0]),
    month: parseInt(parts[1]),
    day: parseInt(parts[2])
  });
};

const isCurrentDate = (targetDate: DateMessage, currentDate: DateMessage) => {
  return currentDate.year === targetDate.year
    && currentDate.month === targetDate.month
    && currentDate.day === targetDate.day;
};

const resolveTeamIdToName = (teamId: number) => {
  return getSeasonStore().worldState["teams"][teamId]["name"];
};

onMounted(async () => {
  await getSeasonStore().fetchLatestWorldState();
  await getSeasonStore().fetchSeason();
});
</script>

<template>
  <main>
    <div class="schedule-grid">
      <div
          class="schedule-day"
          v-for="[dateStr, scheduled_matches] of getSeasonStore().season"
          :class="[isCurrentDate(getDateFromDateStr(dateStr), getSeasonStore().currentDate) ? 'is-current-day' : '']"
      >
        <p class="schedule-day-date">{{getDateFromDateStr(dateStr).day}}</p>
        <p v-for="match in scheduled_matches">{{resolveTeamIdToName(match!.awayTeamId!)}} @ {{resolveTeamIdToName(match!.homeTeamId!)}}</p>
      </div>
    </div>
  </main>
</template>

<style scoped>
main {
  overflow: scroll;
}

.schedule-grid {
  display: grid;
  padding: 0 10%;
  grid-template-columns: repeat(5, 1fr);
  grid-auto-flow: row;
}

.schedule-day {
  border-style: solid;
  border-width: 1px;
}

.is-current-day {
  border-color: yellow;
}

.schedule-day-date {
  text-align: left;
  vertical-align: top;
}
</style>