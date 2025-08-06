import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import ScheduleView from "@/views/ScheduleView.vue";
import StatsView from '@/views/StatsView.vue'
import CombatantView from '@/views/CombatantView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
      {
          path: '/',
          name: 'home',
          component: HomeView,
      },
      {
          path: '/schedule',
          name: 'schedule',
          component: ScheduleView,
      },
      {
          path: '/stats',
          name: 'stats',
          component: StatsView,
      },
      {
          path: '/combatant/:combatantId(\\d+)',
          name: 'combatant',
          component: CombatantView,
      },
      // The below should ALWAYS be last to redirect unknown paths correctly
      {
          path: '/:pathMatch(.*)*',
          redirect: HomeView,
      }
  ]
})

export default router
