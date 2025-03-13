<script setup lang="ts">
  import {onMounted, onUnmounted, onUpdated} from "vue";
  import init, { exit, initializeWithCanvas, loadGameLog } from "@/assets/matchvisualizer.js"
  import {getMatchVisualizerStore} from "@/stores/MatchVisualizer";
  import type {WorldStateResponse} from "%/services/world/world.ts";
  const matchVisualizerStore = getMatchVisualizerStore();

  const props = defineProps([
     "gameLogData"
  ]);

  defineEmits(['close'])

  onMounted(async () => {
    if (!matchVisualizerStore.hasWasmLoaded) {
      const worldStateResponse: WorldStateResponse = (await (await fetch(`api/world_state`)).json());
      matchVisualizerStore.worldStateBytes = worldStateResponse.worldStateJson;
      await init(await fetch("/matchvisualizer_opt.wasm"))
        .catch(err => {
          if (!err.message.startsWith("Using exceptions for control flow,")) {
            throw err;
          }
        });
      matchVisualizerStore.hasWasmLoaded = true;

      try {
        // This will block! Do nothing after this.
        initializeWithCanvas("#match-visualizer");
      }
      catch (ex: any) {
        if (!ex.message.startsWith("Using exceptions for control flow,")) {
          throw ex;
        }
      }
    }
  });

  onUpdated(async () => {
    if (props.gameLogData.length === 0) {
      return;
    }

    loadGameLog(props.gameLogData, matchVisualizerStore.worldStateBytes.valueOf());
  });
</script>

<template>
  <div class="overlay" @click="() => { exit(); $emit('close'); }">
    <div class="modal">
      <canvas id="match-visualizer"></canvas>
    </div>
  </div>
</template>

<style scoped>
  .hidden {
    transform: translateY(-100%);
  }

  .modal {
    display: flex;
    flex-direction: column;

    position: relative;
    left: 5%;
    top: 50%;
    -webkit-transform: translateY(-50%);
    -ms-transform: translateY(-50%);
    transform: translateY(-50%);

    width: 90%;
    max-height: 90vw;
    border-color: rgba(60, 60, 60, 1);
    border-radius: 10px;
    background-color: darkgray;
    z-index: 10;
  }

  .overlay {
    display: block;
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.5);
  }

  #match-visualizer {
    padding: 5px;
    margin: 0 auto;
    max-width: 100%;

    /* Disable selecting the canvas */
    -webkit-touch-callout: none;
    -webkit-user-select: none;
    -khtml-user-select: none;
    -moz-user-select: none;
    -ms-user-select: none;
    user-select: none;
    outline: none;
    -webkit-tap-highlight-color: rgba(255, 255, 255, 0); /* mobile webkit */
  }
</style>