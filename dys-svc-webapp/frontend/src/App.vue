<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router';
import GameCarousel from './components/GameCarousel.vue';
import MatchVisualizer from './components/MatchVisualizer.vue';

import { getMatchVisualizerStore } from '@/stores/MatchVisualizer';
import Auth from '@/components/Auth.vue';
import Ticker from '@/components/Ticker.vue';
const matchVisualizerStore = getMatchVisualizerStore();
</script>

<template>
    <header>
        <div class="wrapper">
            <GameCarousel />
            <nav>
                <RouterLink to="/"><h1>DAX</h1></RouterLink>
                <RouterLink to="/schedule"><h1>Schedule</h1></RouterLink>
                <RouterLink to="/stats"><h1>Stats</h1></RouterLink>
                <Auth></Auth>
            </nav>
        </div>
    </header>

    <RouterView />

    <MatchVisualizer
        :gameLogData="matchVisualizerStore.gameLogData"
        @close="matchVisualizerStore.$reset()"
        :class="{ hidden: matchVisualizerStore.gameLogData.length == 0 }"
    />

    <Ticker />
</template>

<style>
nav {
    display: flex;
}

nav a {
    margin: min(10px, 5%);
    display: block;
    font-family: 'Teko';
    font-size: 28px;
}

RouterView {
    /* Same size as ticker
   * ZJ-TODO: set once?
   */
    margin-bottom: min(6%, 30px);
}

h1 {
    font-size: 48px;
}

h2 {
    font-size: 32px;
}
</style>
