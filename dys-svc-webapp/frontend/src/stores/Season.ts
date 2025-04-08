import {type Ref, ref} from 'vue'
import { defineStore } from 'pinia'
import {MatchResponse_MatchSummary as MatchSummary} from "%/services/match_results/summary.ts";
import {WorldStateResponse} from "%/services/world/world.ts";
import type {GetSeasonResponse, MatchInstance} from "%/services/world/schedule.ts";
import {DateMessage} from "%/common/date.ts";

const dateToStr = (date: DateMessage) => {
  return `${date.year}-${date.month.valueOf()}-${date.day}`;
};

const getDateFromDateStr = (dateStr: string): DateMessage => {
  const parts = dateStr.split("-");
  return DateMessage.fromJSON({
    year: parseInt(parts[0]),
    month: parseInt(parts[1]),
    day: parseInt(parts[2])
  });
};

export const getSeasonStore = defineStore('season', () => {
  /// Sorted by date, such that the first entry is chronologically before the next
  const matchesByDate: Ref<Map<string, MatchSummary[]>> = ref(new Map());
  const worldState: Ref<any> = ref({});
  const season: Ref<Map<string, MatchInstance[]>> = ref(new Map());
  const currentDate: Ref<DateMessage> = ref(DateMessage.create());

  const fetchMatchSummaries = async () => {

  }

  const fetchSeason = async () => {
    const seasonResponse: GetSeasonResponse = await(await fetch("/api/season")).json();

    currentDate.value = seasonResponse.currentDate!;
    const matches = seasonResponse
        .allSeries
        .map((series, _1, _2) => series.matches)
        .flat();

    season.value.clear();
    for (const match of matches) {
      const dateStr = dateToStr(match.date!);
      if (season.value.has(dateStr)) {
        season.value.get(dateStr)!.push(match);
      } else {
        season.value.set(dateStr, [match]);
      }
    }

    season.value = new Map(
        [...season.value.entries()].sort(([dateStrA, _a], [dateStrB, _b]) => {
          const dateA = getDateFromDateStr(dateStrA);
          const dateB = getDateFromDateStr(dateStrB);
          if (dateA.year != dateB.year) {
            return dateA.year - dateB.year;
          }
          if (dateA.month != dateB.month) {
            return dateA.month - dateB.month;
          }

          return dateA.day - dateB.day;
        })
    );
  };

  const fetchLatestWorldState = async () => {
    const response: WorldStateResponse = await (await fetch("/api/world_state")).json();
    const responseObject = JSON.parse(String.fromCharCode(...response.worldStateJson!));
    worldState.value = {};
    for (const key in responseObject) {
      // ZJ-TODO: handle world state values that aren't arrays
      //          we don't have those yet
      worldState.value[key] = {};
      const values = responseObject[key];

      for (const value of values) {
        worldState.value[key][value["id"]] = value;
      }
    }
  };

  return { matchesByDate, worldState, season, currentDate, fetchLatestWorldState, fetchSeason };
})
