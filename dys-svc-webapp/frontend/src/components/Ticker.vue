<script setup lang="ts">
const tickerStrings = [
    "League commissioner Erickson XIII searching for ways to \"reduce, or otherwise make less gross\" the dismembering of combatants",
    "As part of their sponsorship of the month, Arguscorp is offering 25% off all life insurance policies and 50% higher payouts on workplace injuries",
    "DAX lead physicist: \"we're also unsure why the balls explode\"",
];
</script>

<template>
  <div class="container">
    <ul class="ticker">
      <li v-for="tickerItem in tickerStrings">{{tickerItem}}</li>
      <li> <!-- empty item to have an additional separator after our list --> </li>
    </ul>
  </div>
</template>

<style scoped>
.container {
  display: flex;
  overflow-x: hidden;
  position: fixed;
  height: min(6%, 30px);
  min-width: 100%;
  background-color: rgba(0, 0, 0, 0.9);
  bottom: 0;
}

.ticker {
  display: flex;
  align-items: center;
  flex: 1 0 max-content;
  gap: 1rem;
  min-width: 100%;
  width: 100%;
  margin-left: 100%;

  animation-name: ticker-scroll;
  animation-duration: 40s;
  animation-timing-function: linear;
  animation-delay: 0s;
  animation-iteration-count: infinite;
  animation-play-state: running;
  animation-direction: normal;

  -webkit-animation-name: ticker-scroll;
  -webkit-animation-duration: 40s;
  -webkit-animation-timing-function: linear;
  -webkit-animation-delay: 0s;
  -webkit-animation-iteration-count: infinite;
  -webkit-animation-play-state: running;
  -webkit-animation-direction: normal;
}

.ticker li {
  padding-right: 5px;
}

@keyframes ticker-scroll {
  0% {
    transform: translateX(0);
  }
  100% {
    transform: translateX(calc(-100cqw - 100%));
  }
}
</style>