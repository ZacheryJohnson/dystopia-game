<script setup lang="ts">
  import {ref} from "vue";
  import GameCarouselElement from "./GameCarouselElement.vue";

  const games = ref([
    {
      game_id: 1,
      away_team_abbreviation: "FOO",
      home_team_abbreviation: "BAR",
      away_team_score: 3,
      home_team_score: 9,
    },    
    {
      game_id: 2,
      away_team_abbreviation: "QUUX",
      home_team_abbreviation: "BAZ",
      away_team_score: 12,
      home_team_score: 2,
    }
  ]);
</script>

<template>
  <div class="carousel-frame" v-if="games.length > 0">
    <GameCarouselElement
      v-for="game in games"
      :key="game.game_id"
      :away_abbr="game.away_team_abbreviation"
      :home_abbr="game.home_team_abbreviation"
      :away_score="game.away_team_score"
      :home_score="game.home_team_score"
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