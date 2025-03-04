<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import GameCarousel from './components/GameCarousel.vue'
import MatchVisualizer from './components/MatchVisualizer.vue';

import { getMatchVisualizerStore } from '@/stores/MatchVisualizer'
const matchVisualizerStore = getMatchVisualizerStore();
</script>

<template>
  <header>
    <div class="wrapper">
      <GameCarousel />
      <nav>
        <RouterLink to="/"><h1>DAX</h1></RouterLink>
        <RouterLink to="/combatants"><h1>Combatants</h1></RouterLink>
      </nav>
    </div>
  </header>

  <RouterView />

  <MatchVisualizer
    v-if="matchVisualizerStore.gameLogData.length > 0"
    :gameLogData="matchVisualizerStore.gameLogData"
    @close="matchVisualizerStore.$reset()"
    />
  <div v-else style="text-align: center; width: 50%; margin-left: auto; margin-right: auto">
    <p>DAX is a work in progress project, and many features are not implemented.</p>
    <p>Click a match result at the top to see a visualization of how the match played out.</p>
  </div>
</template>

<style>
body {
  overflow: clip;
}

nav {
  display: flex;
}

nav a {
  margin: min(10px, 5%);
  display: block;
  font-family: "Teko";
  font-size: 28px;
}
</style>