import { ref } from 'vue';
import { defineStore } from 'pinia';

export const getAuthStore = defineStore('auth', () => {
    const cookie = ref(String());

    return { cookie };
});
