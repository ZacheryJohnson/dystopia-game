import { type Ref, ref } from 'vue';
import { defineStore } from 'pinia';
import { GameSummaryResponse_GameSummary as GameSummary } from '%/services/game_results/summary.ts';
import type { GetSeasonResponse, GameInstance } from '%/services/world/schedule.ts';
import { DateMessage } from '%/common/date.ts';
import type { World } from '%/rust_types/World.ts';
import { fetchApi } from '@/utils.ts';

const dateToStr = (date: Array<any>) => {
    // ZJ-TODO: yuck
    return `${date[2]}-${date[0]}-${date[1]}`;
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
    const gamesByDate: Ref<Map<string, any>> = ref(new Map());
    const gamesById: Ref<Map<number, GameSummary>> = ref(new Map());
    const gameIdToScheduledTime: Ref<Map<number, Date>> = ref(new Map());
    const worldState: Ref<World> = ref({ combatants: [], teams: [] });
    const season: Ref<Map<string, GameInstance[]>> = ref(new Map());
    const stats: Ref<Map<number, Stats>> = ref(new Map());
    const currentDate: Ref<DateMessage> = ref(DateMessage.create());

    const fetchMatchSummaries = async () => {};

    const fetchSeason = async () => {
        const seasonResponse = await (await fetchApi('schedule')).json();

        for (const gameId in seasonResponse['game_id_to_scheduled_time_utc']) {
            const timeUtc = seasonResponse['game_id_to_scheduled_time_utc'][gameId];
            gameIdToScheduledTime.value.set(parseInt(gameId), new Date(timeUtc));
        }

        currentDate.value = seasonResponse['current_date'];
        const series: [[GameInstance]] = seasonResponse['series']
            // @ts-ignore
            .map((series, _1, _2) => series.games)
            .flat();

        season.value.clear();
        for (const games of series) {
            for (const game of Object.values(games)) {
                // ZJ-TODO: below
                // @ts-ignore
                const dateStr = dateToStr(game['date']);
                if (season.value.has(dateStr)) {
                    season.value.get(dateStr)!.push(game);
                } else {
                    season.value.set(dateStr, [game]);
                }
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
        const response = await (await fetchApi('world/state')).json();
        const world_state = response['world_state_json'];
        const world: World = JSON.parse(world_state);

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
        const response = await (
            await fetchApi(`stats/season/${season_id}`)
        ).json();

        const statlineMap: { [key in string]?: Stats } = response['combatant_statlines'];

        for (const [combatantId, statline] of Object.entries(statlineMap)) {
            // ZJ-TODO: handle undefined statline
            stats.value.set(Number(combatantId), statline!);
        }
    };

    return {
        gamesByDate,
        gamesById,
        gameIdToScheduledTime,
        worldState,
        season,
        stats,
        currentDate,
        fetchLatestWorldState,
        fetchSeason,
        fetchSeasonStats,
    };
});
