<script setup lang="ts">
import { onMounted, type Ref, ref } from 'vue'
import { getSeasonStore, type Stats } from '@/stores/Season.ts'
import {
    GetGameStatlinesRequest,
    type GetGameStatlinesResponse
} from '%/services/game_results/stats.ts'
import { useRoute } from 'vue-router'
import { fetchApi } from '@/utils.ts'
import  { date_MonthToJSON, type DateMessage } from '%/common/date.ts'

const route = useRoute()

const combatantId: Ref<number | undefined> = ref();

type GameStatline = {
    date: DateMessage,
    opponent: string,
    stats: Stats,
};
const gameStatlines: Ref<GameStatline[]> = ref([])

const dateToStr = (date: DateMessage) => {
    return `${date_MonthToJSON(date.month)} ${date.day}`;
};

onMounted(async () => {
    combatantId.value = Number(route.params['combatantId']);

    await getSeasonStore().fetchLatestWorldState();
    await getSeasonStore().fetchSeason();
    await getSeasonStore().fetchSeasonStats();

    let latestStatsRequest = GetGameStatlinesRequest.create();
    latestStatsRequest.combatantIds = [combatantId.value];
    latestStatsRequest.numberOfMostRecentGames = 3;

    const latestStatsResponse: GetGameStatlinesResponse = await fetchApi(`game_results/stats`, {
        method: 'POST',
        body: JSON.stringify(latestStatsRequest)
    });

    let statlines: GameStatline[] = [];
    for (const statlineResponse of latestStatsResponse.statlines) {
        const gameId = statlineResponse.gameId;
        const gameSummary = getSeasonStore().gamesById.get(gameId);

        // ZJ-TODO: this sucks
        const selfTeamName = Object.values(getSeasonStore().worldState.teams)
            .find(team => {
                // ZJ-TODO: fix - combatants isn't holding CombatantInstances, just IDs
                // @ts-ignore
                return team?.combatants.find(com => com == combatantId.value) != null;
            })?.name;

        const selfIsAway = selfTeamName == gameSummary!.awayTeamName!;
        const opponentName = selfIsAway ? gameSummary!.homeTeamName! : gameSummary!.awayTeamName!;

        const combatantStatlineRaw = statlineResponse.combatantStatlines[combatantId.value];
        const statline: Stats = JSON.parse(String.fromCharCode(...combatantStatlineRaw));
        statlines.push({
            date: gameSummary!.date!,
            opponent: opponentName,
            stats: statline,
        });
    }

    gameStatlines.value = statlines;
})
</script>

<template>
    <main>
        <h1>
            {{ getSeasonStore().worldState.combatants[combatantId!]?.name || '(name not found)' }}
        </h1>
        <h2>Recent Matches</h2>
        <table>
            <thead>
                <tr>
                    <th>Date</th>
                    <th>Opponent</th>
                    <th>Points</th>
                    <th>Throws</th>
                    <th>Hits</th>
                    <th>Shoves</th>
                </tr>
            </thead>
            <tbody>
                <tr v-for="statline in gameStatlines">
                    <td>{{ dateToStr(statline.date) }}</td>
                    <td>{{ statline.opponent }}</td>
                    <td>{{ statline.stats.points }}</td>
                    <td>{{ statline.stats.throws }}</td>
                    <td>{{ statline.stats.hits }}</td>
                    <td>{{ statline.stats.shoves }}</td>
                </tr>
            </tbody>
        </table>
    </main>
</template>

<style scoped>
main {
    overflow: scroll;
}
</style>