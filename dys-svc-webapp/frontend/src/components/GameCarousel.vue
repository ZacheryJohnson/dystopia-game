<script setup lang="ts">
  import {ref, onMounted, type Ref} from "vue";
  import GameCarouselElement from "./GameCarouselElement.vue";

  // ZJ-TODO: this should be generated from protocol, not defined in both client + server
  type MatchResultT = {
    gameId: number,
    awayTeamAbbreviation: string,
    homeTeamAbbreviation: string,
    awayTeamScore: number,
    homeTeamScore: number,
    gameLogData: Uint8Array,
  };

  const games: Ref<MatchResultT[]> = ref([]);

  onMounted(async () => {
    const match_results = JSON.parse((await (await fetch(`api/latest_games`)).json()))["match_results"];

    games.value = [];
    let gameId = 1;
    for (const match of match_results) {
      const newGame: MatchResultT = {
        gameId: gameId++,
        awayTeamAbbreviation: match["away_team_name"].substring(0, 3).toUpperCase(),
        homeTeamAbbreviation: match["home_team_name"].substring(0, 3).toUpperCase(),
        awayTeamScore: match["away_team_score"],
        homeTeamScore: match["home_team_score"],
        gameLogData: match["game_log_serialized"],
      };

      games.value.push(newGame);
    }
  });
</script>

<template>
  <div class="carousel-frame" v-if="games.length > 0">
    <GameCarouselElement
      v-for="game in games"
      :key="game.gameId"
      :awayAbbr="game.awayTeamAbbreviation"
      :homeAbbr="game.homeTeamAbbreviation"
      :awayScore="game.awayTeamScore"
      :homeScore="game.homeTeamScore"
      :gameLogData="game.gameLogData"
    />
  </div>
  <div class="carousel-frame" v-else>
    <p>No matches! Check back soon.</p>
  </div>
</template>

<style>
.carousel-frame {
  display: grid;
  grid-auto-flow: column;
  grid-auto-columns: minmax(8%, 140px);

  border-bottom: 2px solid;
}
</style>