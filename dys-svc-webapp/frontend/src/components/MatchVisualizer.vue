<script setup lang="ts">
    import { onMounted, onUpdated, ref } from "vue";
    import init, { initializeWithCanvas, loadGameLog } from "@/assets/matchvisualizer.js"

    const isWasmLoaded = ref(false);

    const props = defineProps([
       "gameLogData" 
    ]);

    onMounted(async () => {
        init(await fetch("/matchvisualizer_opt.wasm"))
            .then(() => startGameThen(async () => { 
                isWasmLoaded.value = true;
            }))
            .catch(err => {
                if (!err.message.startsWith("Using exceptions for control flow,")) {
                    throw err;
                }
            });
    });

    onUpdated(() => {
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
    <p v-show="!isWasmLoaded" id="match-visualizer-loading-text">Loading...</p>
    <canvas id="match-visualizer"></canvas>
</template>

<style>
    #match-visualizer-loading-text {
        padding: 0;
        margin: 0 auto;
        display: block;
        text-align: center;
    }

    #match-visualizer {
        padding: 0;
        margin: 0 auto;
        display: block;
        width: 70%;
        max-width: 70%;
        height: 50%;
        max-height: 50%;
        justify-content: center;

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