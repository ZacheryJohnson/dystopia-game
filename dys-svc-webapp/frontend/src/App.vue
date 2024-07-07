<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import GameCarousel from './components/GameCarousel.vue'
import MatchVisualizer from './components/MatchVisualizer.vue';
</script>

<template>
  <header>
    <div class="wrapper">
      <GameCarousel />
      <nav>
        <RouterLink to="/"><h1>DAX</h1></RouterLink>
        <RouterLink to="/"><h1>Schedule</h1></RouterLink>
        <RouterLink to="/"><h1>Teams</h1></RouterLink>
        <RouterLink to="/"><h1>Combatants</h1></RouterLink>
        <RouterLink to="/"><h1>Settings</h1></RouterLink>
      </nav>
    </div>
  </header>

  <RouterView />

  <body>
    <MatchVisualizer />
  </body>
</template>

<style>
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