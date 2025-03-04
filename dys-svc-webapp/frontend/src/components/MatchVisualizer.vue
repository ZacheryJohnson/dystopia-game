<script setup lang="ts">
    import { onMounted } from "vue";
    import init, { exit, initializeWithCanvas, loadGameLog } from "@/assets/matchvisualizer.js"
    import {getMatchVisualizerStore} from "@/stores/MatchVisualizer";
    const matchVisualizerStore = getMatchVisualizerStore();

    const props = defineProps([
       "gameLogData" 
    ]);

    defineEmits(['close'])

    onMounted(async () => {
      if (!matchVisualizerStore.hasWasmLoaded) {
        await init(await fetch("/matchvisualizer_opt.wasm"))
          .catch(err => {
            if (!err.message.startsWith("Using exceptions for control flow,")) {
              throw err;
            }
          });
        matchVisualizerStore.hasWasmLoaded = true;
        startGameThen(() => {});
      }

      loadGameLog(props.gameLogData);
    });

    // This function will block immediately upon calling.
    // Any logic that should be run after initialization should be passed through a lambda.
    function startGameThen(andThen: () => void = () => {}) {
        setTimeout(andThen, 0);
        
        try {
          initializeWithCanvas("#match-visualizer");
        } catch (ex: any) {
            if (!ex.message.startsWith("Using exceptions for control flow,")) {
                throw ex;
            }
        }
    }
</script>

<template>
  <div class="overlay" @click="() => { $emit('close'); }">
    <div class="modal">
      <canvas id="match-visualizer"></canvas>
    </div>
  </div>
</template>

<style>
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