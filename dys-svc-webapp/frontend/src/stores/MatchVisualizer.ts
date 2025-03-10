import { ref } from 'vue'
import { defineStore } from 'pinia'

export const getMatchVisualizerStore = defineStore('matchVisualizer', () => {
  const gameLogData = ref(new Uint8Array);
  const selectedMatchId = ref(0);
  const worldState = ref(String());
  const hasWasmLoaded = ref(false);

  function $reset() {
    gameLogData.value = new Uint8Array;
  }

  return { gameLogData, hasWasmLoaded, selectedMatchId, worldState, $reset };
})
