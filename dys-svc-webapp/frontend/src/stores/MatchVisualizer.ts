import { ref } from 'vue'
import { defineStore } from 'pinia'

export const getMatchVisualizerStore = defineStore('matchVisualizer', () => {
  const gameLogData = ref(new Uint8Array);
  const hasWasmLoaded = ref(false);

  function $reset() {
    gameLogData.value = new Uint8Array;
  }

  return { gameLogData, hasWasmLoaded, $reset };
})
