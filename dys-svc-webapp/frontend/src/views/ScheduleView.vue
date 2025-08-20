<script setup lang="ts">
import { onMounted } from 'vue';
import { DateMessage } from '%/common/date.ts';
import { getSeasonStore } from '@/stores/Season.ts';
import type { GameInstance } from '%/services/world/schedule.ts';

const getDateFromDateStr = (dateStr: string): DateMessage => {
    const parts = dateStr.split('-');
    return DateMessage.fromJSON({
        year: parseInt(parts[0]),
        month: parseInt(parts[1]),
        day: parseInt(parts[2]),
    });
};

const dateToStr = (date: DateMessage | undefined): string | undefined => {
    if (!date) {
        return undefined;
    }

    return `${date.year}-${date.month.valueOf()}-${date.day}`;
};

const isCurrentDate = (targetDate: DateMessage, currentDate: DateMessage) => {
    return (
        currentDate.year === targetDate.year &&
        currentDate.month === targetDate.month &&
        currentDate.day === targetDate.day
    );
};

const getTeamLogoPath = (teamId: number): string => {
    const teamName = resolveTeamIdToName(teamId).toLowerCase();
    return `/images/teams/team_wip_${teamName}.png`;
};

const resolveTeamIdToName = (teamId: number): string => {
    return getSeasonStore().worldState.teams[teamId]?.name || 'failed to load name';
};

const gameIsFinished = (game: GameInstance): boolean => {
    const gamesOnDate = getSeasonStore().gamesByDate.get(dateToStr(game.date) || '');
    if (!gamesOnDate) {
        return false;
    }

    for (const scheduledGame of gamesOnDate) {
        if (scheduledGame.gameId == game.gameId) {
            return (
                scheduledGame.awayTeamScore != undefined &&
                scheduledGame.awayTeamScore != 0 &&
                scheduledGame.homeTeamScore != undefined &&
                scheduledGame.homeTeamScore != 0
            );
        }
    }

    return false;
};

const getGameScoreString = (gameId: number): string => {
    // ZJ-TODO: add another map to the season store
    //          we should be able to retrieve a game summary by it's ID alone
    const dateAndGames = getSeasonStore().gamesByDate.values();
    for (const games of dateAndGames) {
        for (const game of games) {
            if (game.gameId === gameId) {
                return ` ${game.awayTeamScore || -1} - ${game.homeTeamScore || -1} `;
            }
        }
    }

    return '';
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
                v-for="[dateStr, scheduledGames] of getSeasonStore().season"
                :class="[
                    isCurrentDate(getDateFromDateStr(dateStr), getSeasonStore().currentDate)
                        ? 'is-current-day'
                        : '',
                ]"
            >
                <p class="schedule-day-date">{{ getDateFromDateStr(dateStr).day }}</p>
                <p v-for="game in scheduledGames">
                    <template v-if="gameIsFinished(game)">
                        <img :src="getTeamLogoPath(game.awayTeamId!)" alt="Away Team Logo" />
                        <span>{{ resolveTeamIdToName(game.awayTeamId!) }}</span>
                        <span>{{ getGameScoreString(game.gameId!) }}</span>
                        <span>{{ resolveTeamIdToName(game.homeTeamId!) }}</span>
                        <img :src="getTeamLogoPath(game.homeTeamId!)" alt="Home Team Logo" />
                    </template>
                    <template v-else>
                        {{ resolveTeamIdToName(game!.awayTeamId!) }} @
                        {{ resolveTeamIdToName(game!.homeTeamId!) }}
                    </template>
                </p>
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
