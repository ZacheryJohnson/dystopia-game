<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import GameCarousel from './components/GameCarousel.vue'
import MatchVisualizer from './components/MatchVisualizer.vue';

import { getMatchVisualizerStore } from '@/stores/MatchVisualizer'
import Auth from "@/components/Auth.vue";
import Ticker from "@/components/Ticker.vue";
const matchVisualizerStore = getMatchVisualizerStore();
</script>

<template>
  <header>
    <div class="wrapper">
      <GameCarousel />
      <nav>
        <RouterLink to="/"><h1>DAX</h1></RouterLink>
        <Auth></Auth>
      </nav>
    </div>
  </header>

  <RouterView />

  <MatchVisualizer
    :gameLogData="matchVisualizerStore.gameLogData"
    @close="matchVisualizerStore.$reset()"
    :class="{ 'hidden': matchVisualizerStore.gameLogData.length == 0 }"
  />

  <Ticker/>
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