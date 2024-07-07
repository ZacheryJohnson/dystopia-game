<script setup lang="ts">
  import {ref} from "vue";
  import GameCarouselElement from "./GameCarouselElement.vue";

  const games = ref([
    {
      gameId: 1,
      awayTeamAbbreviation: "FOO",
      homeTeamAbbreviation: "BAR",
      awayTeamScore: 3,
      homeTeamScore: 9,
      gameLogPath: "/game_log1.bin"
    },    
    {
      gameId: 2,
      awayTeamAbbreviation: "QUUX",
      homeTeamAbbreviation: "BAZ",
      awayTeamScore: 12,
      homeTeamScore: 2,
      gameLogPath: "/game_log2.bin"
    }
  ]);
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
      :gameLogPath="game.gameLogPath"
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