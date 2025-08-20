import { type Ref, ref } from 'vue';
import { defineStore } from 'pinia';
import { GameSummaryResponse_GameSummary as GameSummary } from '%/services/game_results/summary.ts';
import { WorldStateResponse } from '%/services/world/world.ts';
import type { GetSeasonResponse, GameInstance } from '%/services/world/schedule.ts';
import { DateMessage } from '%/common/date.ts';
import type { GetSeasonTotalsResponse } from '%/services/game_results/stats.ts';
import type { World } from '%/rust_types/World.ts';
import { fetchApi } from '@/utils.ts';

const dateToStr = (date: DateMessage) => {
    return `${date.year}-${date.month.valueOf()}-${date.day}`;
};

const getDateFromDateStr = (dateStr: string): DateMessage => {
    const parts = dateStr.split('-');
    return DateMessage.fromJSON({
        year: parseInt(parts[0]),
        month: parseInt(parts[1]),
        day: parseInt(parts[2]),
    });
};

export type Stats = {
    points: number;
    throws: number;
    hits: number;
    shoves: number;
};

export const getSeasonStore = defineStore('season', () => {
    /// Sorted by date, such that the first entry is chronologically before the next
    const gamesByDate: Ref<Map<string, GameSummary[]>> = ref(new Map());
    const gamesById: Ref<Map<number, GameSummary>> = ref(new Map());
    const worldState: Ref<World> = ref({ combatants: [], teams: [] });
    const season: Ref<Map<string, GameInstance[]>> = ref(new Map());
    const stats: Ref<Map<number, Stats>> = ref(new Map());
    const currentDate: Ref<DateMessage> = ref(DateMessage.create());

    const fetchMatchSummaries = async () => {};

    const fetchSeason = async () => {
        const seasonResponse: GetSeasonResponse = await (await fetchApi('season')).json();

        currentDate.value = seasonResponse.currentDate!;
        const games = seasonResponse.allSeries.map((series, _1, _2) => series.games).flat();

        season.value.clear();
        for (const game of games) {
            const dateStr = dateToStr(game.date!);
            if (season.value.has(dateStr)) {
                season.value.get(dateStr)!.push(game);
            } else {
                season.value.set(dateStr, [game]);
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
            }),
        );
    };

    const fetchLatestWorldState = async () => {
        const response: WorldStateResponse = await (await fetchApi('world_state')).json();
        let world: World = JSON.parse(String.fromCharCode(...response.worldStateJson!));

        // ZJ-TODO: the world is serialized by the backend as a vector, not a map.
        //          The IDs that are used as keys do not map to combatant/team instance IDs.
        //          The backend should be sending a correct serialization from the get-go
        let correctedWorld: World = { combatants: {}, teams: {} };
        for (let combatant of Object.values(world.combatants)) {
            correctedWorld.combatants[combatant!.id] = combatant;
        }

        for (let team of Object.values(world.teams)) {
            correctedWorld.teams[team!.id] = team;
        }

        worldState.value = correctedWorld;
    };

    const fetchSeasonStats = async () => {
        const season_id: number = 1;
        const response: GetSeasonTotalsResponse = await (
            await fetchApi(`season_stats/${season_id}`)
        ).json();
        for (const [combatantId, statline] of Object.entries(response.combatantStatlines)) {
            const statlines: Stats = JSON.parse(String.fromCharCode(...statline!));
            stats.value.set(Number(combatantId), statlines);
        }
    };

    return {
        gamesByDate,
        gamesById,
        worldState,
        season,
        stats,
        currentDate,
        fetchLatestWorldState,
        fetchSeason,
        fetchSeasonStats,
    };
});
