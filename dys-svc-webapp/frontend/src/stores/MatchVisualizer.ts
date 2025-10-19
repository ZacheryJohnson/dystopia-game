import { ref } from 'vue';
import { defineStore } from 'pinia';
import { fetchApi } from '@/utils.ts';

export const getMatchVisualizerStore = defineStore('matchVisualizer', () => {
    const gameLogData = ref(new Uint8Array());
    const selectedGameId = ref(0);
    const worldStateBytes = ref(new Uint8Array());
    const hasWasmLoaded = ref(false);

    function $reset() {
        selectedGameId.value = 0;
        gameLogData.value = new Uint8Array();
        worldStateBytes.value = new Uint8Array();
    }

    async function getLatestWorldStateBytes() {
        const response = await (await fetchApi('world/state')).json();
        const encoder = new TextEncoder();
        worldStateBytes.value = encoder.encode(response['world_state_json']);
    }

    return { gameLogData, hasWasmLoaded, selectedGameId, worldStateBytes, getLatestWorldStateBytes, $reset };
});
