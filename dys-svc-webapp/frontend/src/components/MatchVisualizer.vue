<script setup lang="ts">
    import { onMounted, ref } from "vue";
    import { initSync, initialize_with_game_log } from "@/assets/matchvisualizer.js"

    const isWasmLoaded = ref(false);

    onMounted(async () => {
        const wasm_file_contents = await (await fetch("/matchvisualizer_opt.wasm")).arrayBuffer();
        const wasm_module = new WebAssembly.Module(wasm_file_contents);
        isWasmLoaded.value = true;
        initSync(wasm_module);

        setTimeout(async () => {
            await init_game();
        }, 200);
    });

    async function init_game() {
        const serialized_game_log = new Uint8Array(await (await fetch("/game_log.bin")).arrayBuffer());
        initialize_with_game_log("#match-visualizer", serialized_game_log);
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